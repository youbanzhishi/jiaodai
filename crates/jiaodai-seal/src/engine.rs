//! Seal engine: orchestrates the full sealing process

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use jiaodai_core::{
    ContentType, EncryptedContent, JiaodaiError, Result, SealCertificate, SealEngine, Sealable,
    Tape, TapeStatus, TriggerCondition, Viewer,
};

use crate::certificate::CertificateManager;
use crate::crypto::{aes256gcm_encrypt, generate_aes_key, sha256_hash};
use crate::event::{SealEvent, SealEventBus};
use crate::hash_store::HashStore;

/// Default implementation of the SealEngine with event bus and hash store
pub struct DefaultSealEngine {
    hash_store: HashStore,
    event_bus: SealEventBus,
}

impl DefaultSealEngine {
    /// Create a new seal engine
    pub fn new() -> Self {
        Self {
            hash_store: HashStore::new(),
            event_bus: SealEventBus::new(),
        }
    }

    /// Get a reference to the hash store
    pub fn hash_store(&self) -> &HashStore {
        &self.hash_store
    }

    /// Get a reference to the event bus
    pub fn event_bus(&self) -> &SealEventBus {
        &self.event_bus
    }

    /// Get a mutable reference to the event bus for subscribing
    pub fn event_bus_mut(&mut self) -> &mut SealEventBus {
        &mut self.event_bus
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

        // Record the content hash
        self.hash_store.record_hash(&tape_id, &content_hash);

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

        // Generate seal certificate
        let certificate = CertificateManager::generate_certificate(
            &tape_id,
            now,
            &content_hash,
            condition,
            viewers,
        );

        // Broadcast events
        let hash_hex = content_hash
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        self.event_bus.broadcast(&SealEvent::TapeSealed {
            tape_id: tape_id.clone(),
            creator_id: creator_id.to_string(),
            content_hash: hash_hex.clone(),
            at: now,
        });
        self.event_bus.broadcast(&SealEvent::HashRecorded {
            tape_id: tape_id.clone(),
            content_hash: hash_hex,
            at: now,
        });
        self.event_bus
            .broadcast(&SealEvent::CertificateGenerated { tape_id, at: now });

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

    #[tokio::test]
    async fn test_seal_records_hash() {
        let engine = DefaultSealEngine::new();
        let content = TextContent::new(b"Test content for hash".to_vec());
        let condition = TriggerCondition::DateTrigger {
            open_at: Utc::now() + chrono::Duration::days(1),
        };

        let (tape, _) = engine
            .seal(&content, condition, vec![Viewer::Anyone], "creator-1")
            .await
            .unwrap();

        // Verify hash was recorded
        let record = engine.hash_store().get_record(&tape.id);
        assert!(record.is_some());
        assert!(!record.unwrap().on_chain);
    }

    #[tokio::test]
    async fn test_seal_certificate_sharing() {
        let engine = DefaultSealEngine::new();
        let content = TextContent::new(b"Shareable content".to_vec());
        let condition = TriggerCondition::DateTrigger {
            open_at: Utc::now() + chrono::Duration::days(1),
        };

        let (_, cert) = engine
            .seal(&content, condition, vec![Viewer::Anyone], "creator-1")
            .await
            .unwrap();

        let share = CertificateManager::generate_share(
            &cert.tape_id,
            crate::certificate::ShareMethod::ShortLink,
        );
        assert!(share.short_link.contains(&cert.tape_id));
    }
}
