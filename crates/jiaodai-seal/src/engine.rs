//! Seal engine: orchestrates the full sealing process

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use jiaodai_core::{
    ContentType, EncryptedContent, JiaodaiError, Result, SealCertificate, SealEngine, Sealable,
    Tape, TapeStatus, TriggerCondition, Viewer,
};

use crate::crypto::{aes256gcm_encrypt, generate_aes_key, sha256_hash};

/// Default implementation of the SealEngine
pub struct DefaultSealEngine;

impl DefaultSealEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultSealEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SealEngine for DefaultSealEngine {
    async fn seal(
        &self,
        content: &dyn Sealable,
        condition: TriggerCondition,
        viewers: Vec<Viewer>,
        creator_id: &str,
    ) -> Result<(Tape, SealCertificate)> {
        let content_hash = content.content_hash();
        let key = generate_aes_key();
        let encrypted = content.encrypt(&key).await?;

        let now = Utc::now();
        let tape_id = Uuid::new_v4().to_string();

        let tape = Tape {
            id: tape_id.clone(),
            creator_id: creator_id.to_string(),
            content_type: content.content_type(),
            encrypted_content: encrypted.ciphertext.clone(),
            content_hash,
            title: None,
            tags: vec![],
            status: TapeStatus::Sealed,
            created_at: now,
            sealed_at: Some(now),
            unsealed_at: None,
        };

        let certificate = SealCertificate {
            tape_id,
            sealed_at: now,
            content_hash,
            chain_tx_hash: None,
            chain_block_number: None,
            trigger_condition: condition,
            viewers,
        };

        Ok((tape, certificate))
    }
}

/// A simple text content that implements Sealable
pub struct TextContent {
    pub text: Vec<u8>,
}

impl TextContent {
    pub fn new(text: impl Into<Vec<u8>>) -> Self {
        Self { text: text.into() }
    }
}

#[async_trait]
impl Sealable for TextContent {
    fn content_hash(&self) -> [u8; 32] {
        sha256_hash(&self.text)
    }

    async fn encrypt(&self, key: &[u8]) -> Result<EncryptedContent> {
        let key_array: [u8; 32] = key
            .try_into()
            .map_err(|_| JiaodaiError::EncryptionError("Key must be 32 bytes".to_string()))?;
        aes256gcm_encrypt(&self.text, &key_array)
    }

    fn content_type(&self) -> ContentType {
        ContentType::Text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiaodai_core::Viewer;

    #[tokio::test]
    async fn test_seal_text_content() {
        let engine = DefaultSealEngine::new();
        let content = TextContent::new(b"Hello from the past!".to_vec());
        let condition = TriggerCondition::DateTrigger {
            open_at: Utc::now() + chrono::Duration::days(365),
        };
        let viewers = vec![Viewer::Anyone];

        let (tape, cert) = engine
            .seal(&content, condition, viewers, "creator-123")
            .await
            .unwrap();

        assert_eq!(tape.status, TapeStatus::Sealed);
        assert_eq!(tape.creator_id, "creator-123");
        assert!(tape.sealed_at.is_some());
        assert_eq!(cert.tape_id, tape.id);
    }
}
