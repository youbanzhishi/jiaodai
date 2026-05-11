//! Seal event bus for broadcasting sealing/unsealing events
//!
//! Architecture rule: events are broadcast, subscribers can be added
//! without modifying core code. Supports notification, audit, analytics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Seal-related event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SealEvent {
    /// A tape was sealed
    TapeSealed {
        tape_id: String,
        creator_id: String,
        content_hash: String,
        at: DateTime<Utc>,
    },
    /// A tape's content hash was recorded for on-chain submission
    HashRecorded {
        tape_id: String,
        content_hash: String,
        at: DateTime<Utc>,
    },
    /// A seal certificate was generated
    CertificateGenerated {
        tape_id: String,
        at: DateTime<Utc>,
    },
    /// A seal certificate was shared (short link / QR code placeholder)
    CertificateShared {
        tape_id: String,
        share_method: String,
        at: DateTime<Utc>,
    },
}

/// Trait for seal event subscribers
pub trait SealEventSubscriber: Send + Sync {
    /// Handle a seal event
    fn on_event(&self, event: &SealEvent);
}

/// Simple event bus that broadcasts seal events to all subscribers
pub struct SealEventBus {
    subscribers: Vec<Box<dyn SealEventSubscriber>>,
}

impl SealEventBus {
    /// Create a new empty event bus
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    /// Subscribe to seal events
    pub fn subscribe(&mut self, subscriber: Box<dyn SealEventSubscriber>) {
        self.subscribers.push(subscriber);
    }

    /// Broadcast an event to all subscribers
    pub fn broadcast(&self, event: &SealEvent) {
        for subscriber in &self.subscribers {
            subscriber.on_event(event);
        }
    }
}

impl Default for SealEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSubscriber {
        events: std::sync::Mutex<Vec<SealEvent>>,
    }

    impl TestSubscriber {
        fn new() -> Self {
            Self {
                events: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    impl SealEventSubscriber for TestSubscriber {
        fn on_event(&self, event: &SealEvent) {
            self.events.lock().unwrap().push(event.clone());
        }
    }

    #[test]
    fn test_seal_event_bus_broadcast() {
        let mut bus = SealEventBus::new();
        bus.subscribe(Box::new(TestSubscriber::new()));
        let event = SealEvent::TapeSealed {
            tape_id: "tape-1".to_string(),
            creator_id: "creator-1".to_string(),
            content_hash: "abc123".to_string(),
            at: Utc::now(),
        };
        bus.broadcast(&event);
    }
}
