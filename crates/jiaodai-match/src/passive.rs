//! Passive registration flow for the crush scenario
//!
//! When A searches for B's phone number and B is not registered,
//! the system sends an invitation SMS to B. When B registers,
//! their account is automatically linked for potential matching.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A pending invitation for passive registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingInvitation {
    /// The phone number hash of the person being invited
    pub phone_hash: String,
    /// The account ID of the person who initiated the search
    pub inviter_account_id: String,
    /// The tape ID that triggered the invitation
    pub tape_id: String,
    /// When the invitation was created
    pub created_at: DateTime<Utc>,
    /// Whether the invitation has been fulfilled
    pub fulfilled: bool,
}

/// Passive registration manager
///
/// Manages the lifecycle of passive registration invitations:
/// 1. A searches for B → B not registered → invitation created → SMS sent
/// 2. B registers → invitation fulfilled → match checking triggered
pub struct PassiveRegistrationManager {
    invitations: std::sync::Mutex<Vec<PendingInvitation>>,
}

impl PassiveRegistrationManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self {
            invitations: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Create an invitation when a search finds an unregistered user
    pub fn create_invitation(
        &self,
        phone_hash: &str,
        inviter_account_id: &str,
        tape_id: &str,
    ) -> PendingInvitation {
        let invitation = PendingInvitation {
            phone_hash: phone_hash.to_string(),
            inviter_account_id: inviter_account_id.to_string(),
            tape_id: tape_id.to_string(),
            created_at: Utc::now(),
            fulfilled: false,
        };
        self.invitations.lock().unwrap().push(invitation.clone());
        invitation
    }

    /// Check for pending invitations for a newly registered phone hash
    pub fn check_pending_invitations(&self, phone_hash: &str) -> Vec<PendingInvitation> {
        let mut invitations = self.invitations.lock().unwrap();
        let matching: Vec<PendingInvitation> = invitations
            .iter()
            .filter(|inv| inv.phone_hash == phone_hash && !inv.fulfilled)
            .cloned()
            .collect();

        // Mark as fulfilled
        for inv in invitations.iter_mut() {
            if inv.phone_hash == phone_hash && !inv.fulfilled {
                inv.fulfilled = true;
            }
        }

        matching
    }

    /// Get all pending (unfulfilled) invitations
    pub fn get_pending(&self) -> Vec<PendingInvitation> {
        let invitations = self.invitations.lock().unwrap();
        invitations
            .iter()
            .filter(|inv| !inv.fulfilled)
            .cloned()
            .collect()
    }
}

impl Default for PassiveRegistrationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::phone_hash;

    #[test]
    fn test_create_and_check_invitation() {
        let manager = PassiveRegistrationManager::new();
        let hash = phone_hash("13800138000");
        manager.create_invitation(&hash, "inviter-1", "tape-1");

        let pending = manager.check_pending_invitations(&hash);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].inviter_account_id, "inviter-1");

        // Should be fulfilled now
        let pending2 = manager.check_pending_invitations(&hash);
        assert!(pending2.is_empty());
    }

    #[test]
    fn test_get_pending() {
        let manager = PassiveRegistrationManager::new();
        let hash1 = phone_hash("13800138000");
        let hash2 = phone_hash("13900139000");
        manager.create_invitation(&hash1, "inviter-1", "tape-1");
        manager.create_invitation(&hash2, "inviter-2", "tape-2");

        let pending = manager.get_pending();
        assert_eq!(pending.len(), 2);
    }
}
