//! Trigger condition checkers

use async_trait::async_trait;
use chrono::Utc;

use jiaodai_core::{
    ConditionState, ConditionType, IdentityClaim, JiaodaiError, Result, TriggerChecker,
    TriggerContext, ViewerVerifier, ViewerType,
};

/// Checks heartbeat-based trigger conditions
pub struct HeartbeatChecker {
    pub timeout_days: u32,
}

#[async_trait]
impl TriggerChecker for HeartbeatChecker {
    fn condition_type(&self) -> ConditionType {
        ConditionType::Heartbeat
    }

    async fn check(&self, context: &TriggerContext) -> ConditionState {
        let last_heartbeat = match context.heartbeat_last_at {
            Some(t) => t,
            None => return ConditionState::Satisfied,
        };

        let elapsed = Utc::now().signed_duration_since(last_heartbeat).num_days();
        if elapsed >= self.timeout_days as i64 {
            ConditionState::Satisfied
        } else {
            ConditionState::Pending
        }
    }

    async fn serialize(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self.timeout_days)
            .map_err(|e| JiaodaiError::SerializationError(e.to_string()))
    }
}

/// Checks date-based trigger conditions
pub struct DateChecker {
    pub open_at: chrono::DateTime<Utc>,
}

#[async_trait]
impl TriggerChecker for DateChecker {
    fn condition_type(&self) -> ConditionType {
        ConditionType::DateTrigger
    }

    async fn check(&self, _context: &TriggerContext) -> ConditionState {
        if Utc::now() >= self.open_at {
            ConditionState::Satisfied
        } else {
            ConditionState::Pending
        }
    }

    async fn serialize(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self.open_at.to_rfc3339())
            .map_err(|e| JiaodaiError::SerializationError(e.to_string()))
    }
}

/// Checks multi-person confirmation conditions
pub struct MultiConfirmerChecker {
    pub threshold: u32,
    pub total: u32,
}

#[async_trait]
impl TriggerChecker for MultiConfirmerChecker {
    fn condition_type(&self) -> ConditionType {
        ConditionType::MultiConfirm
    }

    async fn check(&self, context: &TriggerContext) -> ConditionState {
        let confirmed = context.confirmed_count.unwrap_or(0);
        if confirmed >= self.threshold {
            ConditionState::Satisfied
        } else if confirmed > 0 {
            ConditionState::Partial
        } else {
            ConditionState::Pending
        }
    }

    async fn serialize(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&(self.threshold, self.total))
            .map_err(|e| JiaodaiError::SerializationError(e.to_string()))
    }
}

/// Verifies viewer identity by account ID
pub struct AccountViewerVerifier {
    pub account_id: String,
}

#[async_trait]
impl ViewerVerifier for AccountViewerVerifier {
    fn viewer_type(&self) -> ViewerType {
        ViewerType::Account
    }

    async fn verify(&self, claim: &IdentityClaim) -> bool {
        match claim {
            IdentityClaim::Account { account_id } => account_id == &self.account_id,
            _ => false,
        }
    }
}

/// Verifies viewer identity by phone number hash
pub struct PhoneHashViewerVerifier {
    pub phone_hash: String,
}

#[async_trait]
impl ViewerVerifier for PhoneHashViewerVerifier {
    fn viewer_type(&self) -> ViewerType {
        ViewerType::PhoneHash
    }

    async fn verify(&self, claim: &IdentityClaim) -> bool {
        match claim {
            IdentityClaim::Phone { phone_hash } => phone_hash == &self.phone_hash,
            _ => false,
        }
    }
}

/// Allows anyone to view
pub struct AnyoneViewerVerifier;

#[async_trait]
impl ViewerVerifier for AnyoneViewerVerifier {
    fn viewer_type(&self) -> ViewerType {
        ViewerType::Anyone
    }

    async fn verify(&self, _claim: &IdentityClaim) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_date_checker_future() {
        let checker = DateChecker {
            open_at: Utc::now() + chrono::Duration::days(365),
        };
        let ctx = TriggerContext {
            tape_id: "test".to_string(),
            current_time: Utc::now(),
            heartbeat_last_at: None,
            confirmed_count: None,
            total_confirmers: None,
        };
        assert_eq!(checker.check(&ctx).await, ConditionState::Pending);
    }

    #[tokio::test]
    async fn test_date_checker_past() {
        let checker = DateChecker {
            open_at: Utc::now() - chrono::Duration::days(1),
        };
        let ctx = TriggerContext {
            tape_id: "test".to_string(),
            current_time: Utc::now(),
            heartbeat_last_at: None,
            confirmed_count: None,
            total_confirmers: None,
        };
        assert_eq!(checker.check(&ctx).await, ConditionState::Satisfied);
    }

    #[tokio::test]
    async fn test_multi_confirmer_partial() {
        let checker = MultiConfirmerChecker { threshold: 3, total: 5 };
        let ctx = TriggerContext {
            tape_id: "test".to_string(),
            current_time: Utc::now(),
            heartbeat_last_at: None,
            confirmed_count: Some(1),
            total_confirmers: Some(5),
        };
        assert_eq!(checker.check(&ctx).await, ConditionState::Partial);
    }

    #[tokio::test]
    async fn test_account_viewer_verifier() {
        let verifier = AccountViewerVerifier { account_id: "user-123".to_string() };
        let valid = IdentityClaim::Account { account_id: "user-123".to_string() };
        let invalid = IdentityClaim::Account { account_id: "user-456".to_string() };
        assert!(verifier.verify(&valid).await);
        assert!(!verifier.verify(&invalid).await);
    }

    #[tokio::test]
    async fn test_anyone_viewer_verifier() {
        let verifier = AnyoneViewerVerifier;
        assert!(verifier.verify(&IdentityClaim::Anonymous).await);
    }
}
