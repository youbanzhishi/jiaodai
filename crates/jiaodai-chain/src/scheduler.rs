//! Batch on-chain scheduler
//!
//! Periodically collects new content hashes and submits them
//! on-chain as a Merkle tree root, reducing gas costs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::merkle::MerkleTree;
use crate::engine::DefaultChainEngine;
use jiaodai_core::{BlockchainAttestation, ChainTimestamp, JiaodaiError, Result};

/// A batch of hashes pending on-chain submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRecord {
    /// Batch ID
    pub id: String,
    /// Tape IDs in this batch
    pub tape_ids: Vec<String>,
    /// Content hashes in this batch (hex-encoded)
    pub content_hashes: Vec<String>,
    /// Merkle root (hex-encoded)
    pub merkle_root: Option<String>,
    /// Whether this batch has been submitted on-chain
    pub submitted: bool,
    /// On-chain transaction hash
    pub tx_hash: Option<String>,
    /// On-chain block number
    pub block_number: Option<u64>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Submitted at
    pub submitted_at: Option<DateTime<Utc>>,
}

/// Batch scheduler for on-chain hash submission
pub struct BatchScheduler {
    batches: std::sync::Mutex<Vec<BatchRecord>>,
    max_batch_size: usize,
}

impl BatchScheduler {
    /// Create a new batch scheduler
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            batches: std::sync::Mutex::new(Vec::new()),
            max_batch_size,
        }
    }

    /// Add a hash to the current batch
    pub fn add_hash(&self, tape_id: &str, content_hash: &str) {
        let mut batches = self.batches.lock().unwrap();

        // Find or create an open batch
        let open_batch = batches.iter_mut().find(|b| !b.submitted && b.content_hashes.len() < self.max_batch_size);

        if let Some(batch) = open_batch {
            batch.tape_ids.push(tape_id.to_string());
            batch.content_hashes.push(content_hash.to_string());
        } else {
            let batch = BatchRecord {
                id: uuid::Uuid::new_v4().to_string(),
                tape_ids: vec![tape_id.to_string()],
                content_hashes: vec![content_hash.to_string()],
                merkle_root: None,
                submitted: false,
                tx_hash: None,
                block_number: None,
                created_at: Utc::now(),
                submitted_at: None,
            };
            batches.push(batch);
        }
    }

    /// Submit the current batch on-chain
    ///
    /// Builds a Merkle tree from all hashes in the batch,
    /// submits the root on-chain.
    pub fn submit_batch(&self, chain_engine: &DefaultChainEngine) -> Result<String> {
        let mut batches = self.batches.lock().unwrap();

        let batch = batches.iter_mut().find(|b| !b.submitted && !b.content_hashes.is_empty())
            .ok_or_else(|| JiaodaiError::SerializationError("No pending batch to submit".to_string()))?;

        // Build Merkle tree
        let leaves: Vec<[u8; 32]> = batch.content_hashes.iter()
            .filter_map(|h| {
                let bytes: Vec<u8> = (0..h.len()).step_by(2)
                    .filter_map(|i| u8::from_str_radix(&h[i..i+2.min(h.len()-i)], 16).ok())
                    .collect();
                if bytes.len() == 32 {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&bytes);
                    Some(arr)
                } else {
                    None
                }
            })
            .collect();

        if leaves.is_empty() {
            return Err(JiaodaiError::SerializationError("No valid hashes in batch".to_string()));
        }

        let tree = MerkleTree::new(leaves);
        let root = tree.root();
        let root_hex: String = root.iter().map(|b| format!("{:02x}", b)).collect();

        batch.merkle_root = Some(root_hex.clone());

        // Submit on-chain (currently returns error as chain not yet connected)
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            chain_engine.submit_hash(&root).await
        });

        match result {
            Ok(attestation) => {
                batch.submitted = true;
                batch.tx_hash = Some(attestation.tx_hash);
                batch.block_number = Some(attestation.block_number);
                batch.submitted_at = Some(Utc::now());
                Ok(batch.id.clone())
            }
            Err(_) => {
                // Chain not available — store locally for later submission
                batch.submitted = false;
                Err(JiaodaiError::SerializationError("Chain submission failed, stored locally".to_string()))
            }
        }
    }

    /// Get pending (unsubmitted) batches
    pub fn get_pending_batches(&self) -> Vec<BatchRecord> {
        let batches = self.batches.lock().unwrap();
        batches.iter().filter(|b| !b.submitted).cloned().collect()
    }

    /// Get all batches
    pub fn get_all_batches(&self) -> Vec<BatchRecord> {
        let batches = self.batches.lock().unwrap();
        batches.clone()
    }
}

impl Default for BatchScheduler {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_hash() {
        let scheduler = BatchScheduler::new(100);
        scheduler.add_hash("tape-1", &"a".repeat(64));
        let pending = scheduler.get_pending_batches();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].content_hashes.len(), 1);
    }

    #[test]
    fn test_batch_size_limit() {
        let scheduler = BatchScheduler::new(2);
        scheduler.add_hash("tape-1", &"a".repeat(64));
        scheduler.add_hash("tape-2", &"b".repeat(64));
        scheduler.add_hash("tape-3", &"c".repeat(64)); // Should create new batch
        let pending = scheduler.get_pending_batches();
        assert_eq!(pending.len(), 2);
    }
}
