//! Match engine: bidirectional seal matching

use async_trait::async_trait;
use sha2::{Digest, Sha256};

use jiaodai_core::{MatchEngine, Result};

/// Default implementation of the MatchEngine
pub struct DefaultMatchEngine;

impl DefaultMatchEngine {
    pub fn new() -> Self { Self }
}

impl Default for DefaultMatchEngine {
    fn default() -> Self { Self::new() }
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
        tracing::debug!(tape_id = tape_id, "Checking for mutual match");
        Ok(None)
    }

    async fn register_for_matching(&self, tape_id: &str, target_hash: &str) -> Result<()> {
        tracing::debug!(tape_id = tape_id, target_hash = target_hash, "Registering for match");
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
    async fn test_check_match_returns_none() {
        let engine = DefaultMatchEngine::new();
        let result = engine.check_match("tape-123").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_register_for_matching() {
        let engine = DefaultMatchEngine::new();
        let result = engine.register_for_matching("tape-123", "some-hash").await;
        assert!(result.is_ok());
    }
}
