//! Chain timestamp engine
//!
//! Provides blockchain-backed timestamp proofs for sealed content.
//! Supports both mock (local) and real (L2) chain backends via
//! the ChainTimestamp trait.

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

use jiaodai_core::{BlockchainAttestation, ChainTimestamp, JiaodaiError, Result};

use crate::merkle::MerkleTree;

/// Chain event types for event bus
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChainEvent {
    /// Hash submitted on-chain
    HashSubmitted {
        batch_id: String,
        merkle_root: String,
        tx_hash: String,
        block_number: u64,
        tape_count: usize,
        at: chrono::DateTime<Utc>,
    },
    /// Attestation verified
    AttestationVerified {
        tape_id: String,
        valid: bool,
        at: chrono::DateTime<Utc>,
    },
    /// Batch submission failed
    SubmissionFailed {
        batch_id: String,
        reason: String,
        at: chrono::DateTime<Utc>,
    },
}

/// Trait for chain event subscribers
pub trait ChainEventSubscriber: Send + Sync {
    fn on_event(&self, event: &ChainEvent);
}

/// Simple event bus for chain events
pub struct ChainEventBus {
    subscribers: Vec<Box<dyn ChainEventSubscriber>>,
}

impl ChainEventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, subscriber: Box<dyn ChainEventSubscriber>) {
        self.subscribers.push(subscriber);
    }

    pub fn broadcast(&self, event: &ChainEvent) {
        for sub in &self.subscribers {
            sub.on_event(event);
        }
    }
}

impl Default for ChainEventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// A stored attestation record (for mock chain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationRecord {
    pub id: String,
    pub merkle_root: String,
    pub root_index: u64,
    pub tx_hash: String,
    pub block_number: u64,
    pub timestamp: i64,
    pub network: String,
    pub tape_ids: Vec<String>,
    pub proofs: Vec<StoredProof>,
    pub created_at: chrono::DateTime<Utc>,
}

/// A stored Merkle proof for a specific tape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredProof {
    pub tape_id: String,
    pub content_hash: String,
    pub proof_path: Vec<String>,
}

/// Mock chain engine for development and testing
///
/// Simulates on-chain behavior locally without requiring
/// an actual blockchain connection. Attestations are stored
/// in memory and can be verified.
pub struct MockChainEngine {
    attestations: Mutex<Vec<AttestationRecord>>,
    next_block: Mutex<u64>,
    next_index: Mutex<u64>,
    network: String,
    event_bus: Mutex<ChainEventBus>,
}

impl MockChainEngine {
    pub fn new() -> Self {
        Self {
            attestations: Mutex::new(Vec::new()),
            next_block: Mutex::new(1_000_000),
            next_index: Mutex::new(0),
            network: "mock-l2".to_string(),
            event_bus: Mutex::new(ChainEventBus::new()),
        }
    }

    /// Create with a specific network name
    pub fn with_network(network: &str) -> Self {
        let mut engine = Self::new();
        engine.network = network.to_string();
        engine
    }

    /// Subscribe to chain events
    pub fn subscribe(&self, subscriber: Box<dyn ChainEventSubscriber>) {
        self.event_bus.lock().unwrap().subscribe(subscriber);
    }

    /// Submit a Merkle root as a batch (not part of ChainTimestamp trait)
    ///
    /// This is the primary entry point for batch submission.
    /// It creates attestation records for each tape in the batch.
    pub fn submit_batch(
        &self,
        tape_ids: &[String],
        content_hashes: &[[u8; 32]],
    ) -> Result<Vec<BlockchainAttestation>> {
        if tape_ids.len() != content_hashes.len() {
            return Err(JiaodaiError::SerializationError(
                "tape_ids and content_hashes must have same length".to_string(),
            ));
        }
        if content_hashes.is_empty() {
            return Err(JiaodaiError::SerializationError(
                "Cannot submit empty batch".to_string(),
            ));
        }

        // Build Merkle tree
        let tree = MerkleTree::new(content_hashes.to_vec());
        let root = tree.root();
        let root_hex: String = root.iter().map(|b| format!("{:02x}", b)).collect();

        // Simulate mining a block
        let block_number;
        let root_index;
        {
            let mut bn = self.next_block.lock().unwrap();
            let mut ri = self.next_index.lock().unwrap();
            block_number = *bn;
            root_index = *ri;
            *bn += 1;
            *ri += 1;
        }

        let tx_hash = format!(
            "0xmock_tx_{}",
            Uuid::new_v4().to_string().replace("-", "")[..16].to_string()
        );
        let timestamp = Utc::now().timestamp();

        // Generate proofs for each tape
        let mut stored_proofs = Vec::new();
        let mut attestations = Vec::new();

        for (i, (tape_id, content_hash)) in tape_ids.iter().zip(content_hashes.iter()).enumerate() {
            let proof = tree.proof(i).unwrap();
            let proof_path = proof.path_hex();
            let hash_hex: String = content_hash.iter().map(|b| format!("{:02x}", b)).collect();

            stored_proofs.push(StoredProof {
                tape_id: tape_id.clone(),
                content_hash: hash_hex.clone(),
                proof_path: proof_path.clone(),
            });

            attestations.push(BlockchainAttestation {
                id: Uuid::new_v4().to_string(),
                tape_id: tape_id.clone(),
                content_hash: hash_hex,
                merkle_root: root_hex.clone(),
                merkle_proof: proof_path,
                root_index,
                tx_hash: tx_hash.clone(),
                block_number,
                timestamp,
                network: self.network.clone(),
                created_at: Utc::now(),
            });
        }

        // Store the attestation record
        let record = AttestationRecord {
            id: Uuid::new_v4().to_string(),
            merkle_root: root_hex.clone(),
            root_index,
            tx_hash: tx_hash.clone(),
            block_number,
            timestamp,
            network: self.network.clone(),
            tape_ids: tape_ids.to_vec(),
            proofs: stored_proofs,
            created_at: Utc::now(),
        };

        self.attestations.lock().unwrap().push(record);

        // Broadcast event
        self.event_bus
            .lock()
            .unwrap()
            .broadcast(&ChainEvent::HashSubmitted {
                batch_id: root_hex.clone(),
                merkle_root: root_hex,
                tx_hash: tx_hash.clone(),
                block_number,
                tape_count: tape_ids.len(),
                at: Utc::now(),
            });

        Ok(attestations)
    }

    /// Get all stored attestations
    pub fn get_attestations(&self) -> Vec<AttestationRecord> {
        self.attestations.lock().unwrap().clone()
    }

    /// Find attestation by tape ID
    pub fn find_attestation_by_tape(&self, tape_id: &str) -> Option<BlockchainAttestation> {
        let attestations = self.attestations.lock().unwrap();
        for record in attestations.iter() {
            for proof in &record.proofs {
                if proof.tape_id == tape_id {
                    return Some(BlockchainAttestation {
                        id: record.id.clone(),
                        tape_id: tape_id.to_string(),
                        content_hash: proof.content_hash.clone(),
                        merkle_root: record.merkle_root.clone(),
                        merkle_proof: proof.proof_path.clone(),
                        root_index: record.root_index,
                        tx_hash: record.tx_hash.clone(),
                        block_number: record.block_number,
                        timestamp: record.timestamp,
                        network: record.network.clone(),
                        created_at: record.created_at,
                    });
                }
            }
        }
        None
    }
}

impl Default for MockChainEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChainTimestamp for MockChainEngine {
    async fn submit_hash(&self, hash: &[u8; 32]) -> Result<BlockchainAttestation> {
        // Single hash submission — wraps in a single-leaf batch
        let _hash_hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
        let tape_id = format!("single-{}", Uuid::new_v4());
        let results = self.submit_batch(&[tape_id.clone()], &[*hash])?;
        results.into_iter().next().ok_or_else(|| {
            JiaodaiError::SerializationError("Batch submission produced no results".to_string())
        })
    }

    async fn verify_attestation(&self, attestation: &BlockchainAttestation) -> Result<bool> {
        // Verify by checking if we have a matching record
        let records = self.attestations.lock().unwrap();
        let found = records.iter().any(|r| {
            r.merkle_root == attestation.merkle_root
                && r.tx_hash == attestation.tx_hash
                && r.block_number == attestation.block_number
        });

        if found {
            self.event_bus
                .lock()
                .unwrap()
                .broadcast(&ChainEvent::AttestationVerified {
                    tape_id: attestation.tape_id.clone(),
                    valid: true,
                    at: Utc::now(),
                });
        }

        Ok(found)
    }
}

/// Timestamp verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampVerification {
    /// Whether the tape has an on-chain timestamp
    pub on_chain: bool,
    /// The tape ID
    pub tape_id: String,
    /// The content hash (hex)
    pub content_hash: Option<String>,
    /// The Merkle root (hex)
    pub merkle_root: Option<String>,
    /// Transaction hash
    pub tx_hash: Option<String>,
    /// Block number
    pub block_number: Option<u64>,
    /// Timestamp
    pub timestamp: Option<i64>,
    /// Network
    pub network: Option<String>,
    /// Merkle proof (hex path)
    pub merkle_proof: Vec<String>,
}

impl TimestampVerification {
    /// Create a verification result indicating no on-chain proof
    pub fn not_found(tape_id: &str) -> Self {
        Self {
            on_chain: false,
            tape_id: tape_id.to_string(),
            content_hash: None,
            merkle_root: None,
            tx_hash: None,
            block_number: None,
            timestamp: None,
            network: None,
            merkle_proof: vec![],
        }
    }

    /// Create a verification result from an attestation
    pub fn from_attestation(att: &BlockchainAttestation) -> Self {
        Self {
            on_chain: true,
            tape_id: att.tape_id.clone(),
            content_hash: Some(att.content_hash.clone()),
            merkle_root: Some(att.merkle_root.clone()),
            tx_hash: Some(att.tx_hash.clone()),
            block_number: Some(att.block_number),
            timestamp: Some(att.timestamp),
            network: Some(att.network.clone()),
            merkle_proof: att.merkle_proof.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_chain_submit_single() {
        let engine = MockChainEngine::new();
        let result = engine.submit_hash(&[42u8; 32]).await;
        assert!(result.is_ok());
        let att = result.unwrap();
        assert_eq!(att.network, "mock-l2");
        assert!(!att.tx_hash.is_empty());
        assert!(att.block_number >= 1_000_000);
    }

    #[tokio::test]
    async fn test_mock_chain_batch_submit() {
        let engine = MockChainEngine::new();
        let tape_ids = vec![
            "tape-1".to_string(),
            "tape-2".to_string(),
            "tape-3".to_string(),
        ];
        let hashes = vec![[1u8; 32], [2u8; 32], [3u8; 32]];

        let results = engine.submit_batch(&tape_ids, &hashes).unwrap();
        assert_eq!(results.len(), 3);

        // All should share the same merkle root and block
        assert_eq!(results[0].merkle_root, results[1].merkle_root);
        assert_eq!(results[0].block_number, results[1].block_number);
    }

    #[tokio::test]
    async fn test_mock_chain_verify() {
        let engine = MockChainEngine::new();
        let att = engine.submit_hash(&[42u8; 32]).await.unwrap();
        let valid = engine.verify_attestation(&att).await.unwrap();
        assert!(valid);
    }

    #[tokio::test]
    async fn test_mock_chain_find_by_tape() {
        let engine = MockChainEngine::new();
        let tape_ids = vec!["tape-1".to_string(), "tape-2".to_string()];
        let hashes = vec![[1u8; 32], [2u8; 32]];

        engine.submit_batch(&tape_ids, &hashes).unwrap();

        let found = engine.find_attestation_by_tape("tape-1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().tape_id, "tape-1");

        let not_found = engine.find_attestation_by_tape("tape-999");
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_timestamp_verification_from_attestation() {
        let engine = MockChainEngine::new();
        let att = engine.submit_hash(&[42u8; 32]).await.unwrap();
        let verification = TimestampVerification::from_attestation(&att);
        assert!(verification.on_chain);
        assert!(verification.merkle_root.is_some());
    }

    #[tokio::test]
    async fn test_timestamp_verification_not_found() {
        let verification = TimestampVerification::not_found("tape-xxx");
        assert!(!verification.on_chain);
    }

    #[test]
    fn test_chain_event_bus() {
        struct TestSub {
            events: Mutex<Vec<ChainEvent>>,
        }
        impl TestSub {
            fn new() -> Self {
                Self {
                    events: Mutex::new(Vec::new()),
                }
            }
        }
        impl ChainEventSubscriber for TestSub {
            fn on_event(&self, event: &ChainEvent) {
                self.events.lock().unwrap().push(event.clone());
            }
        }

        let mut bus = ChainEventBus::new();
        bus.subscribe(Box::new(TestSub::new()));
        bus.broadcast(&ChainEvent::SubmissionFailed {
            batch_id: "test".to_string(),
            reason: "test".to_string(),
            at: Utc::now(),
        });
    }
}
