//! Batch on-chain scheduler
//!
//! Periodically collects new content hashes and submits them
//! on-chain as a Merkle tree root, reducing gas costs.
//! Supports both time-based and threshold-based triggers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::engine::{MockChainEngine, TimestampVerification};
use jiaodai_core::{BlockchainAttestation, JiaodaiError, Result};

/// Configuration for the batch scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Maximum number of hashes per batch before auto-submit (threshold trigger)
    pub max_batch_size: usize,
    /// Minimum number of hashes to submit (skip if fewer)
    pub min_batch_size: usize,
    /// Interval in seconds between automatic submissions (0 = disabled)
    pub auto_submit_interval_secs: u64,
    /// Network name for attestations
    pub network: String,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            min_batch_size: 1,
            auto_submit_interval_secs: 3600, // 1 hour
            network: "mock-l2".to_string(),
        }
    }
}

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
///
/// Collects content hashes from seal operations and batches them
/// for efficient on-chain submission. Supports two trigger modes:
/// - Threshold: auto-submit when batch reaches max_batch_size
/// - Timer: auto-submit at regular intervals (for production use)
pub struct BatchScheduler {
    batches: std::sync::Mutex<Vec<BatchRecord>>,
    config: SchedulerConfig,
    last_submit: std::sync::Mutex<Option<DateTime<Utc>>>,
}

impl BatchScheduler {
    /// Create a new batch scheduler with default config
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            batches: std::sync::Mutex::new(Vec::new()),
            config: SchedulerConfig {
                max_batch_size,
                ..Default::default()
            },
            last_submit: std::sync::Mutex::new(None),
        }
    }

    /// Create with full configuration
    pub fn with_config(config: SchedulerConfig) -> Self {
        Self {
            batches: std::sync::Mutex::new(Vec::new()),
            config,
            last_submit: std::sync::Mutex::new(None),
        }
    }

    /// Get the scheduler config
    pub fn config(&self) -> &SchedulerConfig {
        &self.config
    }

    /// Add a hash to the current batch
    ///
    /// If the batch reaches the threshold, it will be auto-submitted
    /// (but only if a chain engine is provided via submit_batch).
    pub fn add_hash(&self, tape_id: &str, content_hash: &str) {
        let mut batches = self.batches.lock().unwrap();

        // Find or create an open batch
        let open_batch = batches
            .iter_mut()
            .find(|b| !b.submitted && b.content_hashes.len() < self.config.max_batch_size);

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

    /// Check if a batch is ready to submit (threshold reached or timer expired)
    pub fn should_submit(&self) -> bool {
        let batches = self.batches.lock().unwrap();

        // Check if any open batch has reached threshold
        let threshold_reached = batches
            .iter()
            .any(|b| !b.submitted && b.content_hashes.len() >= self.config.max_batch_size);

        if threshold_reached {
            return true;
        }

        // Check timer: if auto_submit_interval_secs > 0 and enough time has passed
        if self.config.auto_submit_interval_secs > 0 {
            let last = self.last_submit.lock().unwrap();
            let has_pending = batches
                .iter()
                .any(|b| !b.submitted && b.content_hashes.len() >= self.config.min_batch_size);

            if has_pending {
                if let Some(last_time) = *last {
                    let elapsed = Utc::now().signed_duration_since(last_time).num_seconds();
                    if elapsed >= self.config.auto_submit_interval_secs as i64 {
                        return true;
                    }
                } else {
                    // Never submitted yet — check if we have enough
                    return batches.iter().any(|b| {
                        !b.submitted && b.content_hashes.len() >= self.config.min_batch_size
                    });
                }
            }
        }

        false
    }

    /// Submit the current batch on-chain using the MockChainEngine
    ///
    /// Builds a Merkle tree from all hashes in the batch,
    /// submits the root on-chain.
    pub fn submit_batch(
        &self,
        chain_engine: &MockChainEngine,
    ) -> Result<Vec<BlockchainAttestation>> {
        let mut batches = self.batches.lock().unwrap();

        let batch = batches
            .iter_mut()
            .find(|b| !b.submitted && !b.content_hashes.is_empty())
            .ok_or_else(|| {
                JiaodaiError::SerializationError("No pending batch to submit".to_string())
            })?;

        // Parse hex hashes to bytes
        let leaves: Vec<[u8; 32]> = batch
            .content_hashes
            .iter()
            .filter_map(|h| {
                if h.len() != 64 {
                    return None;
                }
                let bytes: Vec<u8> = (0..h.len())
                    .step_by(2)
                    .filter_map(|i| u8::from_str_radix(&h[i..i + 2], 16).ok())
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
            return Err(JiaodaiError::SerializationError(
                "No valid hashes in batch".to_string(),
            ));
        }

        // Submit via mock chain engine
        let attestations = chain_engine.submit_batch(&batch.tape_ids, &leaves)?;

        // Update batch record
        if let Some(first_att) = attestations.first() {
            batch.merkle_root = Some(first_att.merkle_root.clone());
            batch.tx_hash = Some(first_att.tx_hash.clone());
            batch.block_number = Some(first_att.block_number);
        }
        batch.submitted = true;
        batch.submitted_at = Some(Utc::now());

        // Update last submit time
        *self.last_submit.lock().unwrap() = Some(Utc::now());

        Ok(attestations)
    }

    /// Force-submit all pending batches regardless of thresholds
    pub fn flush_all(&self, chain_engine: &MockChainEngine) -> Result<Vec<BlockchainAttestation>> {
        let mut all_attestations = Vec::new();

        loop {
            match self.submit_batch(chain_engine) {
                Ok(atts) => {
                    all_attestations.extend(atts);
                }
                Err(JiaodaiError::SerializationError(msg)) if msg.contains("No pending batch") => {
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        Ok(all_attestations)
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

    /// Get pending hash count across all open batches
    pub fn pending_count(&self) -> usize {
        let batches = self.batches.lock().unwrap();
        batches
            .iter()
            .filter(|b| !b.submitted)
            .map(|b| b.content_hashes.len())
            .sum()
    }

    /// Verify a tape's on-chain timestamp
    pub fn verify_tape_timestamp(
        &self,
        chain_engine: &MockChainEngine,
        tape_id: &str,
    ) -> TimestampVerification {
        match chain_engine.find_attestation_by_tape(tape_id) {
            Some(att) => TimestampVerification::from_attestation(&att),
            None => TimestampVerification::not_found(tape_id),
        }
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

    fn make_hex_hash(val: u8) -> String {
        [val; 32].iter().map(|b| format!("{:02x}", b)).collect()
    }

    #[test]
    fn test_add_hash() {
        let scheduler = BatchScheduler::new(100);
        scheduler.add_hash("tape-1", &make_hex_hash(1));
        let pending = scheduler.get_pending_batches();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].content_hashes.len(), 1);
    }

    #[test]
    fn test_batch_size_limit() {
        let scheduler = BatchScheduler::new(2);
        scheduler.add_hash("tape-1", &make_hex_hash(1));
        scheduler.add_hash("tape-2", &make_hex_hash(2));
        scheduler.add_hash("tape-3", &make_hex_hash(3)); // Should create new batch
        let pending = scheduler.get_pending_batches();
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_submit_batch() {
        let scheduler = BatchScheduler::new(100);
        let engine = MockChainEngine::new();

        scheduler.add_hash("tape-1", &make_hex_hash(1));
        scheduler.add_hash("tape-2", &make_hex_hash(2));

        let results = scheduler.submit_batch(&engine).unwrap();
        assert_eq!(results.len(), 2);

        let pending = scheduler.get_pending_batches();
        assert!(pending.is_empty());
    }

    #[test]
    fn test_flush_all() {
        let scheduler = BatchScheduler::new(2);
        let engine = MockChainEngine::new();

        scheduler.add_hash("tape-1", &make_hex_hash(1));
        scheduler.add_hash("tape-2", &make_hex_hash(2));
        scheduler.add_hash("tape-3", &make_hex_hash(3));

        let results = scheduler.flush_all(&engine).unwrap();
        assert_eq!(results.len(), 3);
        assert!(scheduler.get_pending_batches().is_empty());
    }

    #[test]
    fn test_should_submit_threshold() {
        let config = SchedulerConfig {
            max_batch_size: 2,
            min_batch_size: 1,
            auto_submit_interval_secs: 0, // Disable timer for threshold-only test
            network: "mock-l2".to_string(),
        };
        let scheduler = BatchScheduler::with_config(config);
        scheduler.add_hash("tape-1", &make_hex_hash(1));
        assert!(!scheduler.should_submit()); // Only 1, threshold is 2
        scheduler.add_hash("tape-2", &make_hex_hash(2));
        assert!(scheduler.should_submit()); // Reached threshold
    }

    #[test]
    fn test_verify_tape_timestamp() {
        let scheduler = BatchScheduler::new(100);
        let engine = MockChainEngine::new();

        scheduler.add_hash("tape-1", &make_hex_hash(1));
        scheduler.submit_batch(&engine).unwrap();

        let verification = scheduler.verify_tape_timestamp(&engine, "tape-1");
        assert!(verification.on_chain);
        assert_eq!(verification.tape_id, "tape-1");
    }

    #[test]
    fn test_verify_tape_not_found() {
        let scheduler = BatchScheduler::new(100);
        let engine = MockChainEngine::new();

        let verification = scheduler.verify_tape_timestamp(&engine, "tape-xxx");
        assert!(!verification.on_chain);
    }

    #[test]
    fn test_pending_count() {
        let scheduler = BatchScheduler::new(100);
        assert_eq!(scheduler.pending_count(), 0);
        scheduler.add_hash("tape-1", &make_hex_hash(1));
        scheduler.add_hash("tape-2", &make_hex_hash(2));
        assert_eq!(scheduler.pending_count(), 2);
    }

    #[test]
    fn test_custom_config() {
        let config = SchedulerConfig {
            max_batch_size: 50,
            min_batch_size: 5,
            auto_submit_interval_secs: 0,
            network: "arbitrum-sepolia".to_string(),
        };
        let scheduler = BatchScheduler::with_config(config);
        assert_eq!(scheduler.config().max_batch_size, 50);
        assert_eq!(scheduler.config().network, "arbitrum-sepolia");
    }
}
