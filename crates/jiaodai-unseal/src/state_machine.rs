//! Condition state machine for tape lifecycle
//!
//! Reference: Blueprint Chapter 6 (Condition State Machine)
//!
//! States: Created → Sealed → Partial → Triggered → Grace → Unsealed
//! The system NEVER auto-unseals; the final step requires human confirmation.

use jiaodai_core::{JiaodaiError, Result, TapeStatus};

/// Transition a tape status to a new state, validating the transition
pub fn transition_status(current: &TapeStatus, target: TapeStatus) -> Result<TapeStatus> {
    if current.can_transition_to(&target) {
        Ok(target)
    } else {
        Err(JiaodaiError::InvalidStateTransition {
            from: format!("{:?}", current),
            to: format!("{:?}", target),
        })
    }
}

/// Compute the next status based on condition evaluation
pub fn next_status(current: &TapeStatus) -> Vec<TapeStatus> {
    match current {
        TapeStatus::Draft => vec![TapeStatus::Sealed],
        TapeStatus::Sealed => vec![
            TapeStatus::Partial,
            TapeStatus::Triggered,
            TapeStatus::Archived,
        ],
        TapeStatus::Partial => vec![TapeStatus::Sealed],
        TapeStatus::Triggered => vec![TapeStatus::Grace, TapeStatus::Unsealed],
        TapeStatus::Grace => vec![TapeStatus::Unsealed, TapeStatus::Sealed],
        TapeStatus::Unsealed => vec![TapeStatus::Archived],
        TapeStatus::Archived => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transition_draft_to_sealed() {
        let result = transition_status(&TapeStatus::Draft, TapeStatus::Sealed);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TapeStatus::Sealed);
    }

    #[test]
    fn test_valid_transition_sealed_to_triggered() {
        let result = transition_status(&TapeStatus::Sealed, TapeStatus::Triggered);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_transition_triggered_to_grace() {
        let result = transition_status(&TapeStatus::Triggered, TapeStatus::Grace);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_transition_grace_to_unsealed() {
        let result = transition_status(&TapeStatus::Grace, TapeStatus::Unsealed);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_transition_draft_to_unsealed() {
        let result = transition_status(&TapeStatus::Draft, TapeStatus::Unsealed);
        assert!(result.is_err());
    }

    #[test]
    fn test_next_status_sealed() {
        let next = next_status(&TapeStatus::Sealed);
        assert!(next.contains(&TapeStatus::Partial));
        assert!(next.contains(&TapeStatus::Triggered));
        assert!(next.contains(&TapeStatus::Archived));
    }
}
