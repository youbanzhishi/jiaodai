//! Crush (暗恋表白) scene
//!
//! Flow:
//! 1. A searches for B's phone → finds B (or sends invitation if not registered)
//! 2. A creates a sealed tape with condition = MutualMatch(target = B)
//! 3. If B also creates a sealed tape with condition = MutualMatch(target = A):
//!    → Both are notified simultaneously
//! 4. If only A → silence forever (zero information leakage)

use chrono::Utc;
use jiaodai_core::MatchEngine;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use jiaodai_core::Result;
use jiaodai_match::{DefaultMatchEngine, MatchResult, PassiveRegistrationManager, PhoneSearchService, phone_hash};

/// Crush scene service
pub struct CrushScene {
    search_service: PhoneSearchService,
    match_engine: DefaultMatchEngine,
    passive_manager: PassiveRegistrationManager,
}

impl CrushScene {
    /// Create a new crush scene service
    pub fn new() -> Self {
        Self {
            search_service: PhoneSearchService::new(),
            match_engine: DefaultMatchEngine::new(),
            passive_manager: PassiveRegistrationManager::new(),
        }
    }

    /// Search for a phone number (traceless)
    pub fn search_phone(&self, phone: &str) -> CrushSearchResult {
        let result = self.search_service.search(phone);
        CrushSearchResult {
            registered: result.registered,
            phone_hash: result.phone_hash,
            account_id: result.account_id,
        }
    }

    /// Create a crush seal (A→B)
    ///
    /// Returns the tape ID and whether an invitation was sent.
    pub fn create_crush(
        &self,
        creator_account_id: &str,
        creator_phone: &str,
        target_phone: &str,
    ) -> Result<(String, bool)> {
        let target_hash = phone_hash(target_phone);
        let creator_hash = phone_hash(creator_phone);

        // Register creator's phone → account mapping
        self.match_engine.register_phone_account(&creator_hash, creator_account_id);

        // Check if target is registered
        let search_result = self.search_service.search(target_phone);
        let invitation_sent = if !search_result.registered {
            // Create passive registration invitation
            let tape_id = Uuid::new_v4().to_string();
            self.passive_manager.create_invitation(&target_hash, creator_account_id, &tape_id);
            true
        } else {
            false
        };

        // Register for matching using the MatchEngine trait
        let tape_id = Uuid::new_v4().to_string();
        let target_hash_clone = target_hash.clone();
        let tape_id_clone = tape_id.clone();

        // We use a simple async runtime for the trait method
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            self.match_engine.register_for_matching(&tape_id_clone, &target_hash_clone).await
        })?;

        // Check for immediate match
        let match_result = self.match_engine.check_bidirectional_match(&tape_id);
        if match_result.matched {
            tracing::info!(
                tape_id = %tape_id,
                matched_tape_id = ?match_result.matched_tape_id,
                "Mutual match found!"
            );
        }

        Ok((tape_id, invitation_sent))
    }

    /// Called when a new account registers — check for pending invitations
    pub fn on_account_registered(&self, phone: &str, account_id: &str) -> Vec<String> {
        let hash = phone_hash(phone);

        // Register phone → account mapping
        self.search_service.register(phone, account_id);
        self.match_engine.register_phone_account(&hash, account_id);

        // Check pending invitations
        let invitations = self.passive_manager.check_pending_invitations(&hash);
        invitations.iter().map(|inv| inv.tape_id.clone()).collect()
    }

    /// Check if a crush tape has a mutual match
    pub fn check_match(&self, tape_id: &str) -> MatchResult {
        self.match_engine.check_bidirectional_match(tape_id)
    }
}

impl Default for CrushScene {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a crush phone search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrushSearchResult {
    /// Whether the phone is registered
    pub registered: bool,
    /// The phone hash (for matching)
    pub phone_hash: String,
    /// Account ID if registered
    pub account_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crush_search() {
        let scene = CrushScene::new();
        let result = scene.search_phone("13800138000");
        assert!(!result.registered);
    }

    #[test]
    fn test_crush_create_single_direction() {
        let scene = CrushScene::new();
        let (tape_id, invitation_sent) = scene
            .create_crush("account-a", "13800138000", "13900139000")
            .unwrap();
        assert!(!tape_id.is_empty());
        assert!(invitation_sent); // Target not registered
    }

    #[test]
    fn test_passive_registration() {
        let scene = CrushScene::new();

        // A searches for B, B not registered
        let (_tape_id, invitation_sent) = scene
            .create_crush("account-a", "13800138000", "13900139000")
            .unwrap();
        assert!(invitation_sent);

        // B registers
        let related_tapes = scene.on_account_registered("13900139000", "account-b");
        assert!(!related_tapes.is_empty());
    }
}
