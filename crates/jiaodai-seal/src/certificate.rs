//! Seal certificate generation and verification
//!
//! A seal certificate proves that a tape was sealed at a specific time
//! with specific content. It includes:
//! - Content hash (SHA-256)
//! - Seal timestamp
//! - Trigger condition summary
//! - Viewer list
//! - Chain attestation reference (when available)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use jiaodai_core::{JiaodaiError, Result, SealCertificate, TriggerCondition, Viewer};

/// Certificate sharing method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShareMethod {
    ShortLink,
    QrCode,
    Direct,
}

/// A shareable certificate reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateShare {
    /// The tape ID
    pub tape_id: String,
    /// Short link (placeholder, will be OpenLink in Phase 9)
    pub short_link: String,
    /// QR code data (placeholder)
    pub qr_data: String,
    /// Share method
    pub method: ShareMethod,
    /// Created at
    pub created_at: DateTime<Utc>,
}

/// Certificate manager for generating and verifying seal certificates
pub struct CertificateManager;

impl CertificateManager {
    /// Create a new certificate manager
    pub fn new() -> Self {
        Self
    }

    /// Generate a seal certificate from tape data
    pub fn generate_certificate(
        tape_id: &str,
        sealed_at: DateTime<Utc>,
        content_hash: &[u8; 32],
        trigger_condition: TriggerCondition,
        viewers: Vec<Viewer>,
    ) -> SealCertificate {
        SealCertificate {
            tape_id: tape_id.to_string(),
            sealed_at,
            content_hash: *content_hash,
            chain_tx_hash: None,
            chain_block_number: None,
            trigger_condition,
            viewers,
        }
    }

    /// Generate a shareable reference for a certificate
    pub fn generate_share(tape_id: &str, method: ShareMethod) -> CertificateShare {
        CertificateShare {
            tape_id: tape_id.to_string(),
            short_link: format!("https://jiaod.ai/s/{}", tape_id),
            qr_data: format!("jiaodai://tape/{}", tape_id),
            method,
            created_at: Utc::now(),
        }
    }

    /// Verify a certificate's content hash integrity
    pub fn verify_certificate_hash(
        certificate: &SealCertificate,
        expected_hash: &[u8; 32],
    ) -> bool {
        certificate.content_hash == *expected_hash
    }

    /// Compute a certificate fingerprint for quick comparison
    pub fn certificate_fingerprint(certificate: &SealCertificate) -> String {
        let mut hasher = Sha256::new();
        hasher.update(certificate.tape_id.as_bytes());
        hasher.update(certificate.sealed_at.to_rfc3339().as_bytes());
        hasher.update(&certificate.content_hash);
        format!("{:x}", hasher.finalize())
    }

    /// Serialize a certificate for sharing
    pub fn serialize_certificate(certificate: &SealCertificate) -> Result<String> {
        serde_json::to_string(certificate).map_err(|e| {
            JiaodaiError::SerializationError(format!("Certificate serialization failed: {}", e))
        })
    }

    /// Deserialize a certificate from a string
    pub fn deserialize_certificate(data: &str) -> Result<SealCertificate> {
        serde_json::from_str(data).map_err(|e| {
            JiaodaiError::SerializationError(format!("Certificate deserialization failed: {}", e))
        })
    }
}

impl Default for CertificateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

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

    #[test]
    fn test_generate_certificate() {
        let cert = CertificateManager::generate_certificate(
            "tape-123",
            Utc::now(),
            &[42u8; 32],
            TriggerCondition::DateTrigger {
                open_at: Utc::now() + Duration::days(365),
            },
            vec![Viewer::Anyone],
        );
        assert_eq!(cert.tape_id, "tape-123");
    }

    #[test]
    fn test_generate_share() {
        let share = CertificateManager::generate_share("tape-123", ShareMethod::ShortLink);
        assert_eq!(share.tape_id, "tape-123");
        assert_eq!(share.method, ShareMethod::ShortLink);
        assert!(share.short_link.contains("tape-123"));
    }

    #[test]
    fn test_verify_certificate_hash() {
        let cert = make_test_certificate();
        assert!(CertificateManager::verify_certificate_hash(
            &cert,
            &[42u8; 32]
        ));
        assert!(!CertificateManager::verify_certificate_hash(
            &cert, &[0u8; 32]
        ));
    }

    #[test]
    fn test_certificate_fingerprint() {
        let cert = make_test_certificate();
        let fp1 = CertificateManager::certificate_fingerprint(&cert);
        let fp2 = CertificateManager::certificate_fingerprint(&cert);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_serialize_deserialize_certificate() {
        let cert = make_test_certificate();
        let serialized = CertificateManager::serialize_certificate(&cert).unwrap();
        let deserialized = CertificateManager::deserialize_certificate(&serialized).unwrap();
        assert_eq!(deserialized.tape_id, cert.tape_id);
    }
}
