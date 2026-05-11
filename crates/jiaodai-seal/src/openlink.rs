//! OpenLink integration: Seal Certificate as Identity Card + short link sharing
//!
//! A seal certificate becomes an OpenLink Identity Card that can be:
//! - Shared via short links
//! - Verified at a public endpoint
//! - Discovered by other agents via /.well-known/agent.json
//!
//! The Identity Card contains:
//! - Content hash (SHA-256)
//! - Seal timestamp
//! - Trigger condition summary
//! - Chain proof reference (when available)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use jiaodai_core::{JiaodaiError, Result, SealCertificate};

/// An OpenLink Identity Card derived from a seal certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityCard {
    /// The type identifier
    #[serde(rename = "type")]
    pub card_type: String,
    /// The tape ID
    pub tape_id: String,
    /// Content hash (hex-encoded SHA-256)
    pub content_hash: String,
    /// When the tape was sealed
    pub sealed_at: DateTime<Utc>,
    /// Human-readable trigger condition summary
    pub trigger_condition_summary: String,
    /// On-chain proof reference (if available)
    pub chain_proof: Option<ChainProofRef>,
    /// Card fingerprint for quick comparison
    pub fingerprint: String,
    /// Card version
    pub version: String,
}

/// Reference to an on-chain proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainProofRef {
    /// Transaction hash
    pub tx_hash: String,
    /// Block number
    pub block_number: u64,
    /// Merkle root (hex)
    pub merkle_root: String,
    /// Network identifier
    pub network: String,
}

/// A short link for sharing a seal certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortLink {
    /// The tape ID this link points to
    pub tape_id: String,
    /// The short link URL
    pub url: String,
    /// The short code (unique identifier in the URL)
    pub short_code: String,
    /// When the link was created
    pub created_at: DateTime<Utc>,
    /// Expiration time (optional)
    pub expires_at: Option<DateTime<Utc>>,
    /// Access count
    pub access_count: u64,
}

/// Verification result for an Identity Card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the card is valid
    pub valid: bool,
    /// The tape ID
    pub tape_id: String,
    /// Hash integrity check result
    pub hash_valid: bool,
    /// Chain proof verification result (None if no chain proof)
    pub chain_verified: Option<bool>,
    /// Timestamp validity
    pub timestamp_valid: bool,
    /// Verification message
    pub message: String,
}

/// Identity Card manager
pub struct IdentityCardManager {
    /// Base URL for short links
    base_url: String,
    /// Short links store
    short_links: std::sync::Mutex<Vec<ShortLink>>,
}

impl IdentityCardManager {
    /// Create a new manager with the given base URL
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            short_links: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Create with default base URL
    pub fn default_manager() -> Self {
        Self::new("https://jiaod.ai")
    }

    /// Generate an Identity Card from a seal certificate
    pub fn generate_identity_card(&self, certificate: &SealCertificate) -> IdentityCard {
        let content_hash_hex: String = certificate
            .content_hash
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        let trigger_summary = summarize_trigger_condition(&certificate.trigger_condition);
        let fingerprint = compute_fingerprint(certificate);

        let chain_proof = match (&certificate.chain_tx_hash, certificate.chain_block_number) {
            (Some(tx_hash), Some(block_number)) => Some(ChainProofRef {
                tx_hash: tx_hash.clone(),
                block_number,
                merkle_root: content_hash_hex.clone(), // Simplified; in production would be actual root
                network: "ethereum-l2".to_string(),
            }),
            _ => None,
        };

        IdentityCard {
            card_type: "jiaodai-seal-certificate".to_string(),
            tape_id: certificate.tape_id.clone(),
            content_hash: content_hash_hex,
            sealed_at: certificate.sealed_at,
            trigger_condition_summary: trigger_summary,
            chain_proof,
            fingerprint,
            version: "1.0.0".to_string(),
        }
    }

    /// Generate a short link for a tape
    pub fn generate_short_link(
        &self,
        tape_id: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> ShortLink {
        let short_code = generate_short_code(tape_id);
        let url = format!("{}/s/{}", self.base_url, short_code);

        let link = ShortLink {
            tape_id: tape_id.to_string(),
            url,
            short_code,
            created_at: Utc::now(),
            expires_at,
            access_count: 0,
        };

        self.short_links.lock().unwrap().push(link.clone());
        link
    }

    /// Find a short link by short code
    pub fn find_short_link(&self, short_code: &str) -> Option<ShortLink> {
        self.short_links
            .lock()
            .unwrap()
            .iter()
            .find(|l| l.short_code == short_code)
            .cloned()
    }

    /// Find short links by tape ID
    pub fn find_links_by_tape(&self, tape_id: &str) -> Vec<ShortLink> {
        self.short_links
            .lock()
            .unwrap()
            .iter()
            .filter(|l| l.tape_id == tape_id)
            .cloned()
            .collect()
    }

    /// Increment access count for a short link
    pub fn record_access(&self, short_code: &str) {
        if let Some(link) = self
            .short_links
            .lock()
            .unwrap()
            .iter_mut()
            .find(|l| l.short_code == short_code)
        {
            link.access_count += 1;
        }
    }

    /// Verify an Identity Card
    pub fn verify_identity_card(
        &self,
        card: &IdentityCard,
        certificate: &SealCertificate,
    ) -> VerificationResult {
        // Check hash integrity
        let expected_hash: String = certificate
            .content_hash
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        let hash_valid = card.content_hash == expected_hash;

        // Check tape ID match
        let tape_id_match = card.tape_id == certificate.tape_id;

        // Check timestamp
        let timestamp_valid = (card.sealed_at - certificate.sealed_at).num_seconds().abs() < 2;

        // Check fingerprint
        let expected_fingerprint = compute_fingerprint(certificate);
        let fingerprint_valid = card.fingerprint == expected_fingerprint;

        let valid = hash_valid && tape_id_match && timestamp_valid && fingerprint_valid;

        VerificationResult {
            valid,
            tape_id: card.tape_id.clone(),
            hash_valid,
            chain_verified: None, // Would need chain engine to verify
            timestamp_valid,
            message: if valid {
                "Identity card verified successfully".to_string()
            } else {
                "Identity card verification failed".to_string()
            },
        }
    }

    /// Serialize an Identity Card to JSON
    pub fn serialize_card(card: &IdentityCard) -> Result<String> {
        serde_json::to_string_pretty(card).map_err(|e| {
            JiaodaiError::SerializationError(format!("Card serialization failed: {}", e))
        })
    }

    /// Deserialize an Identity Card from JSON
    pub fn deserialize_card(data: &str) -> Result<IdentityCard> {
        serde_json::from_str(data).map_err(|e| {
            JiaodaiError::SerializationError(format!("Card deserialization failed: {}", e))
        })
    }
}

impl Default for IdentityCardManager {
    fn default() -> Self {
        Self::default_manager()
    }
}

/// Summarize a trigger condition to a human-readable string
fn summarize_trigger_condition(condition: &jiaodai_core::TriggerCondition) -> String {
    match condition {
        jiaodai_core::TriggerCondition::Heartbeat { timeout_days, .. } => {
            format!("heartbeat_timeout_{}d", timeout_days)
        }
        jiaodai_core::TriggerCondition::MutualMatch { .. } => "mutual_match".to_string(),
        jiaodai_core::TriggerCondition::DateTrigger { open_at } => {
            format!("date_trigger_{}", open_at.format("%Y%m%d"))
        }
        jiaodai_core::TriggerCondition::MultiConfirm {
            threshold,
            confirmers,
        } => {
            format!("multi_confirm_{}_of_{}", threshold, confirmers.len())
        }
        jiaodai_core::TriggerCondition::Composite { conditions, logic } => {
            let logic_str = match logic {
                jiaodai_core::LogicOp::And => "AND",
                jiaodai_core::LogicOp::Or => "OR",
            };
            format!("composite_{}_{}conditions", logic_str, conditions.len())
        }
    }
}

/// Compute a fingerprint for a seal certificate
fn compute_fingerprint(certificate: &SealCertificate) -> String {
    let mut hasher = Sha256::new();
    hasher.update(certificate.tape_id.as_bytes());
    hasher.update(certificate.sealed_at.to_rfc3339().as_bytes());
    hasher.update(&certificate.content_hash);
    let result = hasher.finalize();
    // Take first 16 bytes for a compact fingerprint
    result[..16].iter().map(|b| format!("{:02x}", b)).collect()
}

/// Generate a short code from a tape ID
fn generate_short_code(tape_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(tape_id.as_bytes());
    hasher.update(Utc::now().timestamp_millis().to_string().as_bytes());
    let result = hasher.finalize();
    // Take first 8 bytes for a short code
    result[..8].iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use jiaodai_core::{TriggerCondition, Viewer};

    fn make_test_certificate() -> SealCertificate {
        SealCertificate {
            tape_id: "tape-123".to_string(),
            sealed_at: Utc::now(),
            content_hash: [42u8; 32],
            chain_tx_hash: None,
            chain_block_number: None,
            trigger_condition: TriggerCondition::DateTrigger {
                open_at: Utc::now() + Duration::days(365),
            },
            viewers: vec![Viewer::Anyone],
        }
    }

    fn make_test_certificate_with_chain() -> SealCertificate {
        SealCertificate {
            tape_id: "tape-chain".to_string(),
            sealed_at: Utc::now(),
            content_hash: [99u8; 32],
            chain_tx_hash: Some("0xabc123".to_string()),
            chain_block_number: Some(12345),
            trigger_condition: TriggerCondition::Heartbeat {
                timeout_days: 30,
                confirmers: vec![],
            },
            viewers: vec![Viewer::Anyone],
        }
    }

    #[test]
    fn test_generate_identity_card() {
        let manager = IdentityCardManager::default_manager();
        let cert = make_test_certificate();
        let card = manager.generate_identity_card(&cert);

        assert_eq!(card.card_type, "jiaodai-seal-certificate");
        assert_eq!(card.tape_id, "tape-123");
        assert_eq!(card.trigger_condition_summary, "date_trigger_20270511");
        assert!(card.chain_proof.is_none());
    }

    #[test]
    fn test_identity_card_with_chain_proof() {
        let manager = IdentityCardManager::default_manager();
        let cert = make_test_certificate_with_chain();
        let card = manager.generate_identity_card(&cert);

        assert!(card.chain_proof.is_some());
        let proof = card.chain_proof.unwrap();
        assert_eq!(proof.tx_hash, "0xabc123");
        assert_eq!(proof.block_number, 12345);
    }

    #[test]
    fn test_generate_short_link() {
        let manager = IdentityCardManager::new("https://jiaod.ai");
        let link = manager.generate_short_link("tape-123", None);

        assert!(link.url.starts_with("https://jiaod.ai/s/"));
        assert_eq!(link.tape_id, "tape-123");
        assert!(link.expires_at.is_none());
        assert_eq!(link.access_count, 0);
    }

    #[test]
    fn test_find_short_link() {
        let manager = IdentityCardManager::default_manager();
        let link = manager.generate_short_link("tape-123", None);
        let found = manager.find_short_link(&link.short_code);
        assert!(found.is_some());
        assert_eq!(found.unwrap().tape_id, "tape-123");
    }

    #[test]
    fn test_record_access() {
        let manager = IdentityCardManager::default_manager();
        let link = manager.generate_short_link("tape-123", None);
        manager.record_access(&link.short_code);
        manager.record_access(&link.short_code);

        let found = manager.find_short_link(&link.short_code).unwrap();
        assert_eq!(found.access_count, 2);
    }

    #[test]
    fn test_verify_identity_card() {
        let manager = IdentityCardManager::default_manager();
        let cert = make_test_certificate();
        let card = manager.generate_identity_card(&cert);
        let result = manager.verify_identity_card(&card, &cert);

        assert!(result.valid);
        assert!(result.hash_valid);
        assert!(result.timestamp_valid);
    }

    #[test]
    fn test_verify_identity_card_tampered() {
        let manager = IdentityCardManager::default_manager();
        let cert = make_test_certificate();
        let mut card = manager.generate_identity_card(&cert);
        card.content_hash = "tampered".to_string();

        let result = manager.verify_identity_card(&card, &cert);
        assert!(!result.valid);
        assert!(!result.hash_valid);
    }

    #[test]
    fn test_serialize_deserialize_card() {
        let manager = IdentityCardManager::default_manager();
        let cert = make_test_certificate();
        let card = manager.generate_identity_card(&cert);

        let serialized = IdentityCardManager::serialize_card(&card).unwrap();
        let deserialized = IdentityCardManager::deserialize_card(&serialized).unwrap();

        assert_eq!(deserialized.tape_id, card.tape_id);
        assert_eq!(deserialized.content_hash, card.content_hash);
    }

    #[test]
    fn test_summarize_trigger_conditions() {
        assert_eq!(
            summarize_trigger_condition(&TriggerCondition::Heartbeat {
                timeout_days: 30,
                confirmers: vec![],
            }),
            "heartbeat_timeout_30d"
        );
        assert_eq!(
            summarize_trigger_condition(&TriggerCondition::MutualMatch {
                target_account_id: "acc-1".to_string(),
            }),
            "mutual_match"
        );
        assert_eq!(
            summarize_trigger_condition(&TriggerCondition::MultiConfirm {
                threshold: 3,
                confirmers: vec![
                    jiaodai_core::Confirmer {
                        account_id: None,
                        phone_hash: None,
                        name: "a".to_string(),
                        last_confirmed_at: None,
                    };
                    5
                ],
            }),
            "multi_confirm_3_of_5"
        );
    }

    #[test]
    fn test_find_links_by_tape() {
        let manager = IdentityCardManager::default_manager();
        manager.generate_short_link("tape-1", None);
        manager.generate_short_link("tape-1", None);
        manager.generate_short_link("tape-2", None);

        let links = manager.find_links_by_tape("tape-1");
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_short_link_with_expiry() {
        let manager = IdentityCardManager::default_manager();
        let expires = Utc::now() + Duration::days(7);
        let link = manager.generate_short_link("tape-123", Some(expires));
        assert!(link.expires_at.is_some());
    }
}
