//! Identity verification (real-name authentication)
//!
//! Provides interfaces for OCR-based ID scanning and liveness detection.
//! Current implementation is a mock; real implementations would integrate
//! with services like Alibaba Cloud OCR, Tencent Cloud FaceID, etc.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use jiaodai_core::JiaodaiError;

/// Identity verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityVerificationResult {
    /// Whether the verification passed
    pub verified: bool,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Verification method used
    pub method: String,
    /// Timestamp of verification
    pub verified_at: DateTime<Utc>,
}

/// OCR scan result for an ID card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    /// Name extracted from ID card
    pub name: String,
    /// ID number extracted from ID card
    pub id_number: String,
    /// OCR confidence (0.0 - 1.0)
    pub confidence: f64,
}

/// Liveness detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessResult {
    /// Whether the user is a real person (not a photo/video)
    pub is_live: bool,
    /// Confidence score
    pub confidence: f64,
}

/// Identity verification provider trait
///
/// Implement this trait to integrate with a real identity verification service.
#[async_trait]
pub trait IdentityProvider: Send + Sync {
    /// Perform OCR on an ID card image
    async fn ocr_scan(&self, image_data: &[u8]) -> Result<OcrResult, JiaodaiError>;

    /// Perform liveness detection
    async fn liveness_check(&self, video_data: &[u8]) -> Result<LivenessResult, JiaodaiError>;

    /// Verify that a person matches their ID card
    async fn verify_identity(
        &self,
        name: &str,
        id_number: &str,
        face_image: &[u8],
    ) -> Result<IdentityVerificationResult, JiaodaiError>;

    /// Get the provider name
    fn provider_name(&self) -> &str;
}

/// Mock identity verification provider
pub struct MockIdentityProvider {
    /// Always returns verified if true
    pub always_pass: bool,
}

impl MockIdentityProvider {
    /// Create a new mock provider that always passes
    pub fn new_pass() -> Self {
        Self { always_pass: true }
    }

    /// Create a new mock provider that always fails
    pub fn new_fail() -> Self {
        Self { always_pass: false }
    }
}

impl Default for MockIdentityProvider {
    fn default() -> Self {
        Self::new_pass()
    }
}

#[async_trait]
impl IdentityProvider for MockIdentityProvider {
    async fn ocr_scan(&self, _image_data: &[u8]) -> Result<OcrResult, JiaodaiError> {
        if self.always_pass {
            Ok(OcrResult {
                name: "张三".to_string(),
                id_number: "110101199001011234".to_string(),
                confidence: 0.99,
            })
        } else {
            Err(JiaodaiError::SerializationError("OCR scan failed (mock)".to_string()))
        }
    }

    async fn liveness_check(&self, _video_data: &[u8]) -> Result<LivenessResult, JiaodaiError> {
        Ok(LivenessResult {
            is_live: self.always_pass,
            confidence: if self.always_pass { 0.99 } else { 0.1 },
        })
    }

    async fn verify_identity(
        &self,
        _name: &str,
        _id_number: &str,
        _face_image: &[u8],
    ) -> Result<IdentityVerificationResult, JiaodaiError> {
        Ok(IdentityVerificationResult {
            verified: self.always_pass,
            confidence: if self.always_pass { 0.99 } else { 0.1 },
            method: "mock".to_string(),
            verified_at: Utc::now(),
        })
    }

    fn provider_name(&self) -> &str {
        "mock"
    }
}

/// Hash an ID number for privacy-preserving storage
pub fn id_number_hash(id_number: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(id_number.as_bytes());
    hasher.update(b"jiaodai-identity-salt-v1");
    format!("{:x}", hasher.finalize())
}

/// Hash a name for privacy-preserving storage
pub fn name_hash(name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    hasher.update(b"jiaodai-name-salt-v1");
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_identity_pass() {
        let provider = MockIdentityProvider::new_pass();
        let ocr = provider.ocr_scan(&[]).await.unwrap();
        assert!(ocr.confidence > 0.9);

        let liveness = provider.liveness_check(&[]).await.unwrap();
        assert!(liveness.is_live);

        let verify = provider.verify_identity("张三", "110101199001011234", &[]).await.unwrap();
        assert!(verify.verified);
    }

    #[tokio::test]
    async fn test_mock_identity_fail() {
        let provider = MockIdentityProvider::new_fail();
        let liveness = provider.liveness_check(&[]).await.unwrap();
        assert!(!liveness.is_live);
    }

    #[test]
    fn test_id_number_hash_consistency() {
        let h1 = id_number_hash("110101199001011234");
        let h2 = id_number_hash("110101199001011234");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_id_number_hash_different() {
        let h1 = id_number_hash("110101199001011234");
        let h2 = id_number_hash("110101199001011235");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_name_hash_consistency() {
        let h1 = name_hash("张三");
        let h2 = name_hash("张三");
        assert_eq!(h1, h2);
    }
}
