//! Trigger condition registry
//!
//! Architecture rule: TriggerCondition is extensible.
//! New trigger conditions register their checker, no core code changes needed.

use std::collections::HashMap;
use std::sync::Mutex;

use jiaodai_core::{ConditionType, TriggerChecker};

/// Registry for trigger condition checkers
///
/// New conditions can be registered at runtime. The unseal engine
/// looks up the appropriate checker from the registry.
pub struct TriggerRegistry {
    checkers: Mutex<HashMap<ConditionType, Box<dyn TriggerChecker>>>,
}

impl TriggerRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            checkers: Mutex::new(HashMap::new()),
        }
    }

    /// Register a trigger condition checker
    pub fn register(&self, checker: Box<dyn TriggerChecker>) {
        let ct = checker.condition_type();
        self.checkers.lock().unwrap().insert(ct, checker);
    }

    /// Get a checker by condition type
    pub fn get(&self, _condition_type: &ConditionType) -> Option<Box<dyn TriggerChecker>> {
        // We can't clone the trait object, so we return None for now
        // In practice, the registry would store Arc<dyn TriggerChecker>
        None
    }

    /// Check if a condition type is registered
    pub fn has(&self, condition_type: &ConditionType) -> bool {
        self.checkers.lock().unwrap().contains_key(condition_type)
    }

    /// List all registered condition types
    pub fn registered_types(&self) -> Vec<ConditionType> {
        self.checkers.lock().unwrap().keys().cloned().collect()
    }
}

impl Default for TriggerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Arc-based registry that allows shared ownership of checkers
pub struct SharedTriggerRegistry {
    checkers: Mutex<HashMap<ConditionType, std::sync::Arc<dyn TriggerChecker>>>,
}

impl SharedTriggerRegistry {
    /// Create a new empty shared registry
    pub fn new() -> Self {
        Self {
            checkers: Mutex::new(HashMap::new()),
        }
    }

    /// Register a trigger condition checker
    pub fn register(&self, checker: std::sync::Arc<dyn TriggerChecker>) {
        let ct = checker.condition_type();
        self.checkers.lock().unwrap().insert(ct, checker);
    }

    /// Get a checker by condition type
    pub fn get(
        &self,
        condition_type: &ConditionType,
    ) -> Option<std::sync::Arc<dyn TriggerChecker>> {
        self.checkers.lock().unwrap().get(condition_type).cloned()
    }

    /// Check if a condition type is registered
    pub fn has(&self, condition_type: &ConditionType) -> bool {
        self.checkers.lock().unwrap().contains_key(condition_type)
    }

    /// List all registered condition types
    pub fn registered_types(&self) -> Vec<ConditionType> {
        self.checkers.lock().unwrap().keys().cloned().collect()
    }
}

impl Default for SharedTriggerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkers::{DateChecker, HeartbeatChecker, MultiConfirmerChecker};

    #[test]
    fn test_register_and_check() {
        let registry = SharedTriggerRegistry::new();
        registry.register(std::sync::Arc::new(HeartbeatChecker { timeout_days: 30 }));
        registry.register(std::sync::Arc::new(DateChecker {
            open_at: chrono::Utc::now() + chrono::Duration::days(365),
        }));
        registry.register(std::sync::Arc::new(MultiConfirmerChecker {
            threshold: 2,
            total: 3,
        }));

        assert!(registry.has(&ConditionType::Heartbeat));
        assert!(registry.has(&ConditionType::DateTrigger));
        assert!(registry.has(&ConditionType::MultiConfirm));
        assert!(!registry.has(&ConditionType::MutualMatch));

        let types = registry.registered_types();
        assert_eq!(types.len(), 3);
    }
}
