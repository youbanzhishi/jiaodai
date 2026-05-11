//! Unseal event bus for broadcasting unsealing events
//!
//! Events are broadcast at each stage of the unsealing process:
//! condition triggered → grace period → unsealed → viewer notified.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use jiaodai_core::TapeStatus;

/// Unseal-related event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnsealEvent {
    /// A trigger condition was evaluated
    ConditionChecked {
        tape_id: String,
        satisfied: bool,
        condition_type: String,
        at: DateTime<Utc>,
    },
    /// A tape status transitioned
    StatusTransitioned {
        tape_id: String,
        from: TapeStatus,
        to: TapeStatus,
        at: DateTime<Utc>,
    },
    /// A tape entered grace period
    GracePeriodStarted {
        tape_id: String,
        grace_until: DateTime<Utc>,
        at: DateTime<Utc>,
    },
    /// A tape was unsealed
    TapeUnsealed { tape_id: String, at: DateTime<Utc> },
    /// A viewer was notified about an unsealed tape
    ViewerNotified {
        tape_id: String,
        viewer_type: String,
        at: DateTime<Utc>,
    },
    /// A heartbeat was received
    HeartbeatReceived {
        account_id: String,
        at: DateTime<Utc>,
    },
    /// A mutual match was found
    MatchFound {
        tape_id_a: String,
        tape_id_b: String,
        at: DateTime<Utc>,
    },
    /// A confirmer confirmed
    ConfirmerConfirmed {
        tape_id: String,
        confirmer_id: String,
        confirmed_count: u32,
        threshold: u32,
        at: DateTime<Utc>,
    },
}

/// Trait for unseal event subscribers
pub trait UnsealEventSubscriber: Send + Sync {
    /// Handle an unseal event
    fn on_event(&self, event: &UnsealEvent);
}

/// Simple event bus that broadcasts unseal events
pub struct UnsealEventBus {
    subscribers: Vec<Box<dyn UnsealEventSubscriber>>,
}

impl UnsealEventBus {
    /// Create a new empty event bus
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    /// Subscribe to unseal events
    pub fn subscribe(&mut self, subscriber: Box<dyn UnsealEventSubscriber>) {
        self.subscribers.push(subscriber);
    }

    /// Broadcast an event to all subscribers
    pub fn broadcast(&self, event: &UnsealEvent) {
        for subscriber in &self.subscribers {
            subscriber.on_event(event);
        }
    }
}

impl Default for UnsealEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSubscriber {
        events: std::sync::Mutex<Vec<UnsealEvent>>,
    }

    impl TestSubscriber {
        fn new() -> Self {
            Self {
                events: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    impl UnsealEventSubscriber for TestSubscriber {
        fn on_event(&self, event: &UnsealEvent) {
            self.events.lock().unwrap().push(event.clone());
        }
    }

    #[test]
    fn test_unseal_event_bus_broadcast() {
        let mut bus = UnsealEventBus::new();
        bus.subscribe(Box::new(TestSubscriber::new()));
        let event = UnsealEvent::TapeUnsealed {
            tape_id: "tape-1".to_string(),
            at: Utc::now(),
        };
        bus.broadcast(&event);
    }
}
