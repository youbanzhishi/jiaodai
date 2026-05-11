//! Core traits for the Jiaodai platform
//!
//! Reference: Blueprint Chapter 3 (Architecture Design - Core Traits)

use async_trait::async_trait;

use crate::models::*;
use crate::Result;

/// Sealable: Content that can be sealed
///
/// This trait represents any content that can be encrypted and sealed.
/// The content produces a hash for integrity verification and can be
/// encrypted with a given key.
#[async_trait]
pub trait Sealable: Send + Sync {
    /// Compute the SHA-256 content hash
    fn content_hash(&self) -> [u8; 32];

    /// Encrypt the content with the given key (AES-256-GCM)
    async fn encrypt(&self, key: &[u8]) -> Result<EncryptedContent>;

    /// Get the content type
    fn content_type(&self) -> ContentType;
}

/// TriggerChecker: Check if an unseal condition is met
///
/// Each type of trigger condition implements this trait to provide
/// condition checking logic.
#[async_trait]
pub trait TriggerChecker: Send + Sync {
    /// Get the condition type
    fn condition_type(&self) -> ConditionType;

    /// Check the condition against the given context
    async fn check(&self, context: &TriggerContext) -> ConditionState;

    /// Serialize the condition parameters
    async fn serialize(&self) -> Result<Vec<u8>>;
}

/// ViewerVerifier: Verify a viewer's identity
///
/// Each type of viewer identity implements this trait to verify
/// whether an identity claim matches.
#[async_trait]
pub trait ViewerVerifier: Send + Sync {
    /// Get the viewer type
    fn viewer_type(&self) -> ViewerType;

    /// Verify an identity claim against this viewer
    async fn verify(&self, claim: &IdentityClaim) -> bool;
}

/// SealEngine: The core sealing engine
///
/// Orchestrates the full sealing process: hash, encrypt, generate certificate.
#[async_trait]
pub trait SealEngine: Send + Sync {
    /// Seal content with the given trigger condition and viewers
    async fn seal(
        &self,
        content: &dyn Sealable,
        condition: TriggerCondition,
        viewers: Vec<Viewer>,
        creator_id: &str,
    ) -> Result<(Tape, SealCertificate)>;
}

/// UnsealEngine: The core unsealing engine
///
/// Orchestrates the unsealing process: check conditions, manage state transitions.
#[async_trait]
pub trait UnsealEngine: Send + Sync {
    /// Check if a tape can be unsealed
    async fn check_unseal(&self, tape_id: &str) -> Result<ConditionState>;

    /// Attempt to unseal a tape
    async fn unseal(&self, tape_id: &str, claim: &IdentityClaim) -> Result<Tape>;
}

/// MatchEngine: The mutual-match engine
///
/// Checks for bidirectional seal matches (暗恋表白 scenario).
#[async_trait]
pub trait MatchEngine: Send + Sync {
    /// Check if there's a mutual match for the given tape
    async fn check_match(&self, tape_id: &str) -> Result<Option<String>>;

    /// Register a new tape for match checking
    async fn register_for_matching(&self, tape_id: &str, target_hash: &str) -> Result<()>;
}

/// ChainTimestamp: Blockchain timestamp service
///
/// Provides blockchain-backed timestamp proofs for sealed content.
#[async_trait]
pub trait ChainTimestamp: Send + Sync {
    /// Submit a hash for on-chain timestamping
    async fn submit_hash(&self, hash: &[u8; 32]) -> Result<BlockchainAttestation>;

    /// Verify an on-chain attestation
    async fn verify_attestation(&self, attestation: &BlockchainAttestation) -> Result<bool>;
}
