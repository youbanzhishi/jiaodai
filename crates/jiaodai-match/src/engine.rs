//! Match engine: bidirectional seal matching
//!
//! Core algorithm:
//! 1. A seals a tape with target_hash = hash(B's phone)
//! 2. B seals a tape with target_hash = hash(A's phone)
//! 3. System checks: if hash(B's phone) matches B's account AND
//!    hash(A's phone) matches A's account → mutual match!
//! 4. Only when both directions exist → both notified simultaneously
//! 5. If only one direction → silence (zero information leakage)

use async_trait::async_trait;
use chrono::Utc;
use sha2::{Digest, Sha256};

use jiaodai_core::{JiaodaiError, MatchEngine, Result};

/// A sealed tape registered for matching
#[derive(Debug, Clone)]
struct MatchEntry {
    /// The tape ID
    tape_id: String,
    /// The account ID of the creator
    creator_account_id: String,
    /// The hash of the target's phone number
    target_phone_hash: String,
    /// When the entry was created
    created_at: chrono::DateTime<chrono::Utc>,
}

/// Result of a match check
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Whether a mutual match was found
    pub matched: bool,
    /// The tape ID of the matching tape (if any)
    pub matched_tape_id: Option<String>,
    /// The account ID of the matched person
    pub matched_account_id: Option<String>,
}

/// Default implementation of the MatchEngine with in-memory storage
pub struct DefaultMatchEngine {
    /// Registered match entries
    entries: std::sync::Mutex<Vec<MatchEntry>>,
    /// Phone hash → account ID mapping
    phone_to_account: std::sync::Mutex<Vec<(String, String)>>,
}

impl DefaultMatchEngine {
    /// Create a new match engine
    pub fn new() -> Self {
        Self {
            entries: std::sync::Mutex::new(Vec::new()),
            phone_to_account: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Register a phone → account mapping (from account registration)
    pub fn register_phone_account(&self, phone_hash: &str, account_id: &str) {
        let mut map = self.phone_to_account.lock().unwrap();
        if !map.iter().any(|(h, _)| h == phone_hash) {
            map.push((phone_hash.to_string(), account_id.to_string()));
        }
    }

    /// Check for bidirectional match
    ///
    /// Given tape_id (A→B), check if B→A also exists:
    /// 1. Find A's entry with target_phone_hash = hash(B's phone)
    /// 2. Find B's phone → B's account_id
    /// 3. Check if B has an entry with target_phone_hash = hash(A's phone)
    /// 4. If yes → mutual match!
    pub fn check_bidirectional_match(&self, tape_id: &str) -> MatchResult {
        let entries = self.entries.lock().unwrap();
        let phone_map = self.phone_to_account.lock().unwrap();

        // Find our entry
        let our_entry = match entries.iter().find(|e| e.tape_id == tape_id) {
            Some(e) => e,
            None => {
                return MatchResult {
                    matched: false,
                    matched_tape_id: None,
                    matched_account_id: None,
                }
            }
        };

        // Find target's account ID
        let target_account_id = match phone_map
            .iter()
            .find(|(h, _)| h == &our_entry.target_phone_hash)
        {
            Some((_, id)) => id,
            None => {
                return MatchResult {
                    matched: false,
                    matched_tape_id: None,
                    matched_account_id: None,
                }
            }
        };

        // Find target's phone hash (hash of A's phone)
        let our_phone_hash = match phone_map
            .iter()
            .find(|(_, id)| id == &our_entry.creator_account_id)
        {
            Some((h, _)) => h,
            None => {
                return MatchResult {
                    matched: false,
                    matched_tape_id: None,
                    matched_account_id: None,
                }
            }
        };

        // Check if target has an entry targeting A
        let reverse_entry = entries.iter().find(|e| {
            e.creator_account_id == *target_account_id && e.target_phone_hash == *our_phone_hash
        });

        match reverse_entry {
            Some(their_entry) => MatchResult {
                matched: true,
                matched_tape_id: Some(their_entry.tape_id.clone()),
                matched_account_id: Some(target_account_id.clone()),
            },
            None => MatchResult {
                matched: false,
                matched_tape_id: None,
                matched_account_id: None,
            },
        }
    }
}

impl Default for DefaultMatchEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a hash of a phone number for matching purposes
pub fn phone_hash(phone: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(phone.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[async_trait]
impl MatchEngine for DefaultMatchEngine {
    async fn check_match(&self, tape_id: &str) -> Result<Option<String>> {
        let result = self.check_bidirectional_match(tape_id);
        if result.matched {
            Ok(result.matched_tape_id)
        } else {
            Ok(None)
        }
    }

    async fn register_for_matching(&self, tape_id: &str, target_hash: &str) -> Result<()> {
        // Find creator account from phone map or use tape_id prefix
        let creator_account_id = format!("creator-of-{}", tape_id);
        let entry = MatchEntry {
            tape_id: tape_id.to_string(),
            creator_account_id,
            target_phone_hash: target_hash.to_string(),
            created_at: Utc::now(),
        };
        self.entries.lock().unwrap().push(entry);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_hash_consistency() {
        let hash1 = phone_hash("13800138000");
        let hash2 = phone_hash("13800138000");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_phone_hash_different_phones() {
        let hash1 = phone_hash("13800138000");
        let hash2 = phone_hash("13900139000");
        assert_ne!(hash1, hash2);
    }

    #[tokio::test]
    async fn test_register_for_matching() {
        let engine = DefaultMatchEngine::new();
        let result = engine.register_for_matching("tape-1", "some-hash").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_no_match_single_direction() {
        let engine = DefaultMatchEngine::new();
        // Register A's phone → account
        let hash_a = phone_hash("13800138000");
        let hash_b = phone_hash("13900139000");
        engine.register_phone_account(&hash_a, "account-a");
        engine.register_phone_account(&hash_b, "account-b");

        // A → B (only one direction)
        let entry = MatchEntry {
            tape_id: "tape-ab".to_string(),
            creator_account_id: "account-a".to_string(),
            target_phone_hash: hash_b,
            created_at: Utc::now(),
        };
        engine.entries.lock().unwrap().push(entry);

        let result = engine.check_bidirectional_match("tape-ab");
        assert!(!result.matched);
    }

    #[tokio::test]
    async fn test_bidirectional_match() {
        let engine = DefaultMatchEngine::new();
        let hash_a = phone_hash("13800138000");
        let hash_b = phone_hash("13900139000");
        engine.register_phone_account(&hash_a, "account-a");
        engine.register_phone_account(&hash_b, "account-b");

        // A → B
        let entry_ab = MatchEntry {
            tape_id: "tape-ab".to_string(),
            creator_account_id: "account-a".to_string(),
            target_phone_hash: hash_b.clone(),
            created_at: Utc::now(),
        };
        engine.entries.lock().unwrap().push(entry_ab);

        // B → A
        let entry_ba = MatchEntry {
            tape_id: "tape-ba".to_string(),
            creator_account_id: "account-b".to_string(),
            target_phone_hash: hash_a,
            created_at: Utc::now(),
        };
        engine.entries.lock().unwrap().push(entry_ba);

        // Check from A's perspective
        let result = engine.check_bidirectional_match("tape-ab");
        assert!(result.matched);
        assert_eq!(result.matched_tape_id, Some("tape-ba".to_string()));
        assert_eq!(result.matched_account_id, Some("account-b".to_string()));

        // Check from B's perspective
        let result = engine.check_bidirectional_match("tape-ba");
        assert!(result.matched);
        assert_eq!(result.matched_tape_id, Some("tape-ab".to_string()));
        assert_eq!(result.matched_account_id, Some("account-a".to_string()));
    }
}
