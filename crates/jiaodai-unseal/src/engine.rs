//! Unseal engine: orchestrates the unsealing process
//!
//! Implements the full state machine: Draft → Sealed → Triggered → Grace → Unsealed
//! Supports all four trigger conditions through the registry pattern.

use async_trait::async_trait;
use chrono::Utc;
use jiaodai_core::{
    ConditionState, IdentityClaim, JiaodaiError, LogicOp, Result, Tape, TapeStatus,
    TriggerCondition, TriggerContext, TriggerChecker, UnsealEngine,
};

use crate::checkers::{DateChecker, HeartbeatChecker, MultiConfirmerChecker};
use crate::event::{UnsealEvent, UnsealEventBus};
use crate::registry::SharedTriggerRegistry;
use crate::state_machine::transition_status;

/// Configuration for the unseal engine
#[derive(Debug, Clone)]
pub struct UnsealConfig {
    /// Grace period in days (default: 7)
    pub grace_period_days: u32,
}

impl Default for UnsealConfig {
    fn default() -> Self {
        Self {
            grace_period_days: 7,
        }
    }
}

/// Default implementation of the UnsealEngine with full state machine
pub struct DefaultUnsealEngine {
    config: UnsealConfig,
    registry: SharedTriggerRegistry,
    event_bus: UnsealEventBus,
    /// In-memory tape status store (production: SQLite)
    tape_statuses: std::sync::Mutex<Vec<(String, TapeStatus)>>,
}

impl DefaultUnsealEngine {
    /// Create a new unseal engine with default configuration
    pub fn new() -> Self {
        let registry = SharedTriggerRegistry::new();
        // Register default checkers
        registry.register(std::sync::Arc::new(HeartbeatChecker { timeout_days: 30 }));
        registry.register(std::sync::Arc::new(DateChecker {
            open_at: chrono::DateTime::<chrono::Utc>::MAX_UTC,
        }));
        registry.register(std::sync::Arc::new(MultiConfirmerChecker { threshold: 2, total: 3 }));

        Self {
            config: UnsealConfig::default(),
            registry,
            event_bus: UnsealEventBus::new(),
            tape_statuses: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: UnsealConfig) -> Self {
        let mut engine = Self::new();
        engine.config = config;
        engine
    }

    /// Get the event bus for subscribing
    pub fn event_bus(&self) -> &UnsealEventBus {
        &self.event_bus
    }

    /// Get mutable reference to event bus
    pub fn event_bus_mut(&mut self) -> &mut UnsealEventBus {
        &mut self.event_bus
    }

    /// Get the trigger registry
    pub fn registry(&self) -> &SharedTriggerRegistry {
        &self.registry
    }

    /// Set a tape's status in the in-memory store
    pub fn set_tape_status(&self, tape_id: &str, status: TapeStatus) {
        let mut statuses = self.tape_statuses.lock().unwrap();
        if let Some(entry) = statuses.iter_mut().find(|(id, _)| id == tape_id) {
            entry.1 = status;
        } else {
            statuses.push((tape_id.to_string(), status));
        }
    }

    /// Get a tape's status from the in-memory store
    pub fn get_tape_status(&self, tape_id: &str) -> Option<TapeStatus> {
        let statuses = self.tape_statuses.lock().unwrap();
        statuses.iter().find(|(id, _)| id == tape_id).map(|(_, s)| s.clone())
    }

    /// Check a trigger condition against a context
    pub async fn check_condition(
        &self,
        condition: &TriggerCondition,
        context: &TriggerContext,
    ) -> ConditionState {
        let condition_type = condition.condition_type();
        let result = match condition {
            TriggerCondition::Heartbeat { timeout_days, .. } => {
                let checker = HeartbeatChecker { timeout_days: *timeout_days };
                checker.check(context).await
            }
            TriggerCondition::MutualMatch { .. } => {
                // Mutual match requires external match engine
                ConditionState::Pending
            }
            TriggerCondition::DateTrigger { open_at } => {
                let checker = DateChecker { open_at: *open_at };
                checker.check(context).await
            }
            TriggerCondition::MultiConfirm { threshold, .. } => {
                let total = context.total_confirmers.unwrap_or(0);
                let checker = MultiConfirmerChecker { threshold: *threshold, total };
                checker.check(context).await
            }
            TriggerCondition::Composite { conditions, logic } => {
                // Flatten composite: check each leaf condition, combine results
                let mut results = Vec::new();
                for cond in conditions {
                    let state = self.check_single_condition(cond, context).await;
                    results.push(state);
                }
                Self::combine_results(&results, logic)
            }
        };

        self.event_bus.broadcast(&UnsealEvent::ConditionChecked {
            tape_id: context.tape_id.clone(),
            satisfied: result == ConditionState::Satisfied,
            condition_type: format!("{:?}", condition_type),
            at: Utc::now(),
        });

        result
    }

    /// Check a single (non-composite) condition
    async fn check_single_condition(
        &self,
        condition: &TriggerCondition,
        context: &TriggerContext,
    ) -> ConditionState {
        match condition {
            TriggerCondition::Heartbeat { timeout_days, .. } => {
                let checker = HeartbeatChecker { timeout_days: *timeout_days };
                checker.check(context).await
            }
            TriggerCondition::MutualMatch { .. } => ConditionState::Pending,
            TriggerCondition::DateTrigger { open_at } => {
                let checker = DateChecker { open_at: *open_at };
                checker.check(context).await
            }
            TriggerCondition::MultiConfirm { threshold, .. } => {
                let total = context.total_confirmers.unwrap_or(0);
                let checker = MultiConfirmerChecker { threshold: *threshold, total };
                checker.check(context).await
            }
            TriggerCondition::Composite { .. } => {
                // Nested composites not supported at this depth
                ConditionState::Pending
            }
        }
    }

    /// Combine multiple condition results with AND/OR logic
    fn combine_results(results: &[ConditionState], logic: &LogicOp) -> ConditionState {
        match logic {
            LogicOp::And => {
                if results.iter().all(|r| *r == ConditionState::Satisfied) {
                    ConditionState::Satisfied
                } else if results.iter().any(|r| matches!(r, ConditionState::Failed(_))) {
                    ConditionState::Failed("AND condition failed".to_string())
                } else {
                    ConditionState::Partial
                }
            }
            LogicOp::Or => {
                if results.iter().any(|r| *r == ConditionState::Satisfied) {
                    ConditionState::Satisfied
                } else if results.iter().any(|r| *r == ConditionState::Partial) {
                    ConditionState::Partial
                } else {
                    ConditionState::Pending
                }
            }
        }
    }

    /// Attempt a status transition and broadcast event
    pub fn try_transition(&self, tape_id: &str, target: TapeStatus) -> Result<TapeStatus> {
        let current = self.get_tape_status(tape_id)
            .ok_or_else(|| JiaodaiError::TapeNotFound(tape_id.to_string()))?;

        let new_status = transition_status(&current, target)?;

        self.event_bus.broadcast(&UnsealEvent::StatusTransitioned {
            tape_id: tape_id.to_string(),
            from: current.clone(),
            to: new_status.clone(),
            at: Utc::now(),
        });

        // Special events for specific transitions
        match &new_status {
            TapeStatus::Grace => {
                let grace_until = Utc::now() + chrono::Duration::days(self.config.grace_period_days as i64);
                self.event_bus.broadcast(&UnsealEvent::GracePeriodStarted {
                    tape_id: tape_id.to_string(),
                    grace_until,
                    at: Utc::now(),
                });
            }
            TapeStatus::Unsealed => {
                self.event_bus.broadcast(&UnsealEvent::TapeUnsealed {
                    tape_id: tape_id.to_string(),
                    at: Utc::now(),
                });
            }
            _ => {}
        }

        self.set_tape_status(tape_id, new_status.clone());
        Ok(new_status)
    }
}

impl Default for DefaultUnsealEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnsealEngine for DefaultUnsealEngine {
    async fn check_unseal(&self, tape_id: &str) -> Result<ConditionState> {
        tracing::debug!(tape_id = tape_id, "Checking unseal conditions");
        let status = self.get_tape_status(tape_id);
        match status {
            Some(TapeStatus::Unsealed) => Ok(ConditionState::Satisfied),
            Some(TapeStatus::Archived) => Ok(ConditionState::Failed("Tape is archived".to_string())),
            _ => Ok(ConditionState::Pending),
        }
    }

    async fn unseal(&self, tape_id: &str, _claim: &IdentityClaim) -> Result<Tape> {
        let status = self.get_tape_status(tape_id)
            .ok_or_else(|| JiaodaiError::TapeNotFound(tape_id.to_string()))?;

        if status != TapeStatus::Triggered && status != TapeStatus::Grace {
            return Err(JiaodaiError::ConditionNotMet(
                format!("Tape is in {:?} status, cannot unseal", status)
            ));
        }

        // Perform the transition
        self.try_transition(tape_id, TapeStatus::Unsealed)?;

        // In production, this would decrypt and return the tape content
        Err(JiaodaiError::SerializationError(
            "Tape unsealed but content retrieval not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_unseal_unknown_tape() {
        let engine = DefaultUnsealEngine::new();
        let result = engine.check_unseal("nonexistent").await.unwrap();
        assert_eq!(result, ConditionState::Pending);
    }

    #[tokio::test]
    async fn test_check_condition_date_trigger() {
        let engine = DefaultUnsealEngine::new();

        // Past date — should be satisfied
        let ctx = TriggerContext {
            tape_id: "tape-1".to_string(),
            current_time: Utc::now(),
            heartbeat_last_at: None,
            confirmed_count: None,
            total_confirmers: None,
        };
        let condition = TriggerCondition::DateTrigger {
            open_at: Utc::now() - chrono::Duration::days(1),
        };
        let result = engine.check_condition(&condition, &ctx).await;
        assert_eq!(result, ConditionState::Satisfied);

        // Future date — should be pending
        let condition = TriggerCondition::DateTrigger {
            open_at: Utc::now() + chrono::Duration::days(365),
        };
        let result = engine.check_condition(&condition, &ctx).await;
        assert_eq!(result, ConditionState::Pending);
    }

    #[tokio::test]
    async fn test_check_condition_heartbeat() {
        let engine = DefaultUnsealEngine::new();
        let ctx = TriggerContext {
            tape_id: "tape-1".to_string(),
            current_time: Utc::now(),
            heartbeat_last_at: Some(Utc::now() - chrono::Duration::days(31)),
            confirmed_count: None,
            total_confirmers: None,
        };
        let condition = TriggerCondition::Heartbeat {
            timeout_days: 30,
            confirmers: vec![],
        };
        let result = engine.check_condition(&condition, &ctx).await;
        assert_eq!(result, ConditionState::Satisfied);
    }

    #[tokio::test]
    async fn test_check_condition_multi_confirm() {
        let engine = DefaultUnsealEngine::new();
        let ctx = TriggerContext {
            tape_id: "tape-1".to_string(),
            current_time: Utc::now(),
            heartbeat_last_at: None,
            confirmed_count: Some(1),
            total_confirmers: Some(3),
        };
        let condition = TriggerCondition::MultiConfirm {
            threshold: 2,
            confirmers: vec![],
        };
        let result = engine.check_condition(&condition, &ctx).await;
        assert_eq!(result, ConditionState::Partial);
    }

    #[tokio::test]
    async fn test_check_composite_and() {
        let engine = DefaultUnsealEngine::new();
        let ctx = TriggerContext {
            tape_id: "tape-1".to_string(),
            current_time: Utc::now(),
            heartbeat_last_at: None,
            confirmed_count: Some(3),
            total_confirmers: Some(3),
        };

        // Both satisfied
        let condition = TriggerCondition::Composite {
            conditions: vec![
                TriggerCondition::DateTrigger { open_at: Utc::now() - chrono::Duration::days(1) },
                TriggerCondition::MultiConfirm { threshold: 2, confirmers: vec![] },
            ],
            logic: LogicOp::And,
        };
        let result = engine.check_condition(&condition, &ctx).await;
        assert_eq!(result, ConditionState::Satisfied);

        // One pending
        let condition = TriggerCondition::Composite {
            conditions: vec![
                TriggerCondition::DateTrigger { open_at: Utc::now() + chrono::Duration::days(1) },
                TriggerCondition::MultiConfirm { threshold: 2, confirmers: vec![] },
            ],
            logic: LogicOp::And,
        };
        let result = engine.check_condition(&condition, &ctx).await;
        assert_eq!(result, ConditionState::Partial);
    }

    #[tokio::test]
    async fn test_check_composite_or() {
        let engine = DefaultUnsealEngine::new();
        let ctx = TriggerContext {
            tape_id: "tape-1".to_string(),
            current_time: Utc::now(),
            heartbeat_last_at: None,
            confirmed_count: Some(0),
            total_confirmers: Some(3),
        };

        let condition = TriggerCondition::Composite {
            conditions: vec![
                TriggerCondition::DateTrigger { open_at: Utc::now() - chrono::Duration::days(1) },
                TriggerCondition::MultiConfirm { threshold: 2, confirmers: vec![] },
            ],
            logic: LogicOp::Or,
        };
        let result = engine.check_condition(&condition, &ctx).await;
        assert_eq!(result, ConditionState::Satisfied);
    }

    #[tokio::test]
    async fn test_status_transition() {
        let engine = DefaultUnsealEngine::new();
        engine.set_tape_status("tape-1", TapeStatus::Draft);
        let result = engine.try_transition("tape-1", TapeStatus::Sealed).unwrap();
        assert_eq!(result, TapeStatus::Sealed);
    }

    #[tokio::test]
    async fn test_invalid_status_transition() {
        let engine = DefaultUnsealEngine::new();
        engine.set_tape_status("tape-1", TapeStatus::Draft);
        let result = engine.try_transition("tape-1", TapeStatus::Unsealed);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_full_lifecycle() {
        let engine = DefaultUnsealEngine::new();
        engine.set_tape_status("tape-1", TapeStatus::Draft);
        engine.try_transition("tape-1", TapeStatus::Sealed).unwrap();
        engine.try_transition("tape-1", TapeStatus::Triggered).unwrap();
        engine.try_transition("tape-1", TapeStatus::Grace).unwrap();
        engine.try_transition("tape-1", TapeStatus::Unsealed).unwrap();
        assert_eq!(engine.get_tape_status("tape-1"), Some(TapeStatus::Unsealed));
    }
}
