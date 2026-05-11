//! Chain timestamp engine

use async_trait::async_trait;
use jiaodai_core::{BlockchainAttestation, ChainTimestamp, JiaodaiError, Result};

/// Default implementation of the ChainTimestamp service
pub struct DefaultChainEngine;

impl DefaultChainEngine {
    pub fn new() -> Self { Self }
}

impl Default for DefaultChainEngine {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl ChainTimestamp for DefaultChainEngine {
    async fn submit_hash(
        &self,
        _hash: &[u8; 32],
    ) -> Result<BlockchainAttestation> {
        // Phase 1: placeholder — returns error
        // Phase 8: will interact with L2 contract via ethers-rs
        Err(JiaodaiError::SerializationError(
            "Blockchain integration not yet implemented".to_string(),
        ))
    }

    async fn verify_attestation(
        &self,
        _attestation: &BlockchainAttestation,
    ) -> Result<bool> {
        // Phase 1: placeholder
        // Phase 8: will verify Merkle proof on-chain
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_submit_hash_not_implemented() {
        let engine = DefaultChainEngine::new();
        let result = engine.submit_hash(&[0u8; 32]).await;
        assert!(result.is_err());
    }
}
