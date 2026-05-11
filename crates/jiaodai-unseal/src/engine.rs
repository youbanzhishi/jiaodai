//! Unseal engine: orchestrates the unsealing process

use async_trait::async_trait;
use jiaodai_core::{ConditionState, IdentityClaim, JiaodaiError, Result, Tape, UnsealEngine};

/// Default implementation of the UnsealEngine
pub struct DefaultUnsealEngine;

impl DefaultUnsealEngine {
    pub fn new() -> Self { Self }
}

impl Default for DefaultUnsealEngine {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl UnsealEngine for DefaultUnsealEngine {
    async fn check_unseal(&self, tape_id: &str) -> Result<ConditionState> {
        tracing::debug!(tape_id = tape_id, "Checking unseal conditions");
        Ok(ConditionState::Pending)
    }

    async fn unseal(&self, _tape_id: &str, _claim: &IdentityClaim) -> Result<Tape> {
        Err(JiaodaiError::ConditionNotMet(
            "Unseal engine not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_unseal_returns_pending() {
        let engine = DefaultUnsealEngine::new();
        let result = engine.check_unseal("tape-123").await.unwrap();
        assert_eq!(result, ConditionState::Pending);
    }
}
