//! Event bus for account-related events
//!
//! Architecture rule: events are broadcast, subscribers can be added
//! without modifying core code. Supports notification, audit, analytics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Account-related event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccountEvent {
    /// A new account was registered
    AccountCreated {
        account_id: String,
        phone_hash: String,
        at: DateTime<Utc>,
    },
    /// A phone number was bound to an account
    PhoneBound {
        account_id: String,
        phone_hash: String,
        is_primary: bool,
        at: DateTime<Utc>,
    },
    /// A phone number was unbound from an account
    PhoneUnbound {
        account_id: String,
        phone_hash: String,
        at: DateTime<Utc>,
    },
    /// A phone number was changed (old → new)
    PhoneChanged {
        account_id: String,
        old_phone_hash: String,
        new_phone_hash: String,
        at: DateTime<Utc>,
    },
    /// Account logged in
    LoggedIn {
        account_id: String,
        phone_hash: String,
        at: DateTime<Utc>,
    },
    /// Identity verification completed
    IdentityVerified {
        account_id: String,
        at: DateTime<Utc>,
    },
    /// Account recovered via identity verification
    AccountRecovered {
        account_id: String,
        new_phone_hash: String,
        at: DateTime<Utc>,
    },
}

/// Trait for event subscribers
pub trait EventSubscriber: Send + Sync {
    /// Handle an account event
    fn on_event(&self, event: &AccountEvent);
}

/// Simple event bus that broadcasts events to all subscribers
pub struct EventBus {
    subscribers: Vec<Box<dyn EventSubscriber>>,
}

impl EventBus {
    /// Create a new empty event bus
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    /// Subscribe to account events
    pub fn subscribe(&mut self, subscriber: Box<dyn EventSubscriber>) {
        self.subscribers.push(subscriber);
    }

    /// Broadcast an event to all subscribers
    pub fn broadcast(&self, event: &AccountEvent) {
        for subscriber in &self.subscribers {
            subscriber.on_event(event);
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSubscriber {
        events: std::sync::Mutex<Vec<AccountEvent>>,
    }

    impl TestSubscriber {
        fn new() -> Self {
            Self {
                events: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn event_count(&self) -> usize {
            self.events.lock().unwrap().len()
        }
    }

    impl EventSubscriber for TestSubscriber {
        fn on_event(&self, event: &AccountEvent) {
            self.events.lock().unwrap().push(event.clone());
        }
    }

    #[test]
    fn test_event_bus_broadcast() {
        let mut bus = EventBus::new();
        let sub = TestSubscriber::new();
        bus.subscribe(Box::new(TestSubscriber::new()));
        let event = AccountEvent::AccountCreated {
            account_id: "acc-1".to_string(),
            phone_hash: "hash-1".to_string(),
            at: Utc::now(),
        };
        bus.broadcast(&event);
    }

    #[test]
    fn test_event_bus_multiple_subscribers() {
        let mut bus = EventBus::new();
        bus.subscribe(Box::new(TestSubscriber::new()));
        bus.subscribe(Box::new(TestSubscriber::new()));
        let event = AccountEvent::LoggedIn {
            account_id: "acc-1".to_string(),
            phone_hash: "hash-1".to_string(),
            at: Utc::now(),
        };
        bus.broadcast(&event);
    }
}
