//! Content hash storage and verification
//!
//! Manages the storage of content hashes for sealed tapes.
//! Hashes are stored locally and queued for on-chain submission.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use jiaodai_core::JiaodaiError;

/// Helper to convert a 32-byte hash to hex string
fn hash_to_hex(hash: &[u8; 32]) -> String {
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

/// A content hash record, stored locally and queued for on-chain submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashRecord {
    /// The tape ID this hash belongs to
    pub tape_id: String,
    /// The SHA-256 content hash (hex-encoded)
    pub content_hash: String,
    /// Whether this hash has been submitted on-chain
    pub on_chain: bool,
    /// The on-chain transaction hash (if submitted)
    pub chain_tx_hash: Option<String>,
    /// The on-chain block number (if submitted)
    pub chain_block_number: Option<u64>,
    /// When the hash was recorded
    pub recorded_at: DateTime<Utc>,
    /// When the hash was submitted on-chain
    pub submitted_at: Option<DateTime<Utc>>,
}

/// In-memory hash store
///
/// In production, this would be backed by SQLite/PostgreSQL.
pub struct HashStore {
    records: std::sync::Mutex<Vec<HashRecord>>,
}

impl HashStore {
    /// Create a new empty hash store
    pub fn new() -> Self {
        Self {
            records: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Record a content hash for a tape
    pub fn record_hash(&self, tape_id: &str, content_hash: &[u8; 32]) -> HashRecord {
        let record = HashRecord {
            tape_id: tape_id.to_string(),
            content_hash: hash_to_hex(content_hash),
            on_chain: false,
            chain_tx_hash: None,
            chain_block_number: None,
            recorded_at: Utc::now(),
            submitted_at: None,
        };
        self.records.lock().unwrap().push(record.clone());
        record
    }

    /// Mark a hash as submitted on-chain
    pub fn mark_on_chain(
        &self,
        tape_id: &str,
        tx_hash: &str,
        block_number: u64,
    ) -> Result<(), JiaodaiError> {
        let mut records = self.records.lock().unwrap();
        let record = records
            .iter_mut()
            .find(|r| r.tape_id == tape_id)
            .ok_or_else(|| JiaodaiError::TapeNotFound(tape_id.to_string()))?;
        record.on_chain = true;
        record.chain_tx_hash = Some(tx_hash.to_string());
        record.chain_block_number = Some(block_number);
        record.submitted_at = Some(Utc::now());
        Ok(())
    }

    /// Get a hash record by tape ID
    pub fn get_record(&self, tape_id: &str) -> Option<HashRecord> {
        let records = self.records.lock().unwrap();
        records.iter().find(|r| r.tape_id == tape_id).cloned()
    }

    /// Get all hashes that are not yet on-chain
    pub fn get_pending_hashes(&self) -> Vec<HashRecord> {
        let records = self.records.lock().unwrap();
        records.iter().filter(|r| !r.on_chain).cloned().collect()
    }

    /// Verify a content hash against a stored record
    pub fn verify_hash(&self, tape_id: &str, content_hash: &[u8; 32]) -> bool {
        let records = self.records.lock().unwrap();
        let hex = hash_to_hex(content_hash);
        records.iter().any(|r| r.tape_id == tape_id && r.content_hash == hex)
    }
}

impl Default for HashStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_get_hash() {
        let store = HashStore::new();
        let hash = [42u8; 32];
        store.record_hash("tape-1", &hash);
        let record = store.get_record("tape-1").unwrap();
        assert_eq!(record.tape_id, "tape-1");
        assert!(!record.on_chain);
    }

    #[test]
    fn test_mark_on_chain() {
        let store = HashStore::new();
        let hash = [42u8; 32];
        store.record_hash("tape-1", &hash);
        store.mark_on_chain("tape-1", "tx-abc", 12345).unwrap();
        let record = store.get_record("tape-1").unwrap();
        assert!(record.on_chain);
        assert_eq!(record.chain_tx_hash, Some("tx-abc".to_string()));
    }

    #[test]
    fn test_get_pending_hashes() {
        let store = HashStore::new();
        store.record_hash("tape-1", &[1u8; 32]);
        store.record_hash("tape-2", &[2u8; 32]);
        store.mark_on_chain("tape-1", "tx-abc", 12345).unwrap();
        let pending = store.get_pending_hashes();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].tape_id, "tape-2");
    }

    #[test]
    fn test_verify_hash() {
        let store = HashStore::new();
        let hash = [42u8; 32];
        store.record_hash("tape-1", &hash);
        assert!(store.verify_hash("tape-1", &hash));
        assert!(!store.verify_hash("tape-1", &[0u8; 32]));
        assert!(!store.verify_hash("tape-nonexistent", &hash));
    }
}
