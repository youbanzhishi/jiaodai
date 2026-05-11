//! SMS service provider interface
//!
//! Provides a trait for SMS verification code delivery.
//! Currently uses a mock implementation; real SMS provider
//! (e.g., Alibaba Cloud SMS, Tencent Cloud SMS) can be plugged in
//! by implementing the `SmsProvider` trait.

use async_trait::async_trait;

/// SMS service provider trait
///
/// Implement this trait to integrate with a real SMS provider.
/// The mock implementation logs the code instead of sending it.
#[async_trait]
pub trait SmsProvider: Send + Sync {
    /// Send a verification code to the given phone number
    async fn send_verification_code(&self, phone: &str, code: &str) -> SmsResult;

    /// Send an invitation SMS (for passive registration in crush scenario)
    async fn send_invitation(&self, phone: &str, message: &str) -> SmsResult;

    /// Get the provider name for logging
    fn provider_name(&self) -> &str;
}

/// Result of an SMS send operation
#[derive(Debug)]
pub struct SmsResult {
    pub success: bool,
    pub message: String,
    pub request_id: Option<String>,
}

impl SmsResult {
    /// Create a successful result
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            request_id: None,
        }
    }

    /// Create a failed result
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            request_id: None,
        }
    }

    /// Convert to Result<(), SmsResult> for error handling
    pub fn to_result(self) -> Result<(), Self> {
        if self.success {
            Ok(())
        } else {
            Err(self)
        }
    }
}

/// Mock SMS provider that logs codes instead of sending them
pub struct MockSmsProvider;

#[async_trait]
impl SmsProvider for MockSmsProvider {
    async fn send_verification_code(&self, phone: &str, code: &str) -> SmsResult {
        tracing::info!(phone = phone, code = code, "Mock SMS: verification code");
        SmsResult::ok(format!("Mock SMS sent to {}", phone))
    }

    async fn send_invitation(&self, phone: &str, message: &str) -> SmsResult {
        tracing::info!(phone = phone, msg = message, "Mock SMS: invitation");
        SmsResult::ok(format!("Mock invitation sent to {}", phone))
    }

    fn provider_name(&self) -> &str {
        "mock"
    }
}

/// Verification code manager (in-memory mock)
///
/// In production, use Redis with TTL for code storage.
pub struct VerificationCodeManager {
    /// In-memory store: phone → (code, expires_at)
    codes: std::sync::Mutex<Vec<(String, String, chrono::DateTime<chrono::Utc>)>>,
}

impl VerificationCodeManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self {
            codes: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Generate and store a verification code for a phone number
    pub fn generate_code(&self, phone: &str) -> String {
        let code = format!("{:06}", rand::Rng::gen_range(&mut rand::thread_rng(), 0..1000000));
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);
        let mut codes = self.codes.lock().unwrap();
        // Remove any existing code for this phone
        codes.retain(|(p, _, _)| p != phone);
        codes.push((phone.to_string(), code.clone(), expires_at));
        code
    }

    /// Verify a code for a phone number
    pub fn verify_code(&self, phone: &str, code: &str) -> bool {
        let mut codes = self.codes.lock().unwrap();
        let now = chrono::Utc::now();
        if let Some(pos) = codes.iter().position(|(p, c, exp)| p == phone && c == code && exp > &now) {
            codes.remove(pos);
            true
        } else {
            false
        }
    }
}

impl Default for VerificationCodeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_sms_send_code() {
        let provider = MockSmsProvider;
        let result = provider.send_verification_code("13800138000", "123456").await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_mock_sms_send_invitation() {
        let provider = MockSmsProvider;
        let result = provider.send_invitation("13800138000", "Someone has a message for you").await;
        assert!(result.success);
    }

    #[test]
    fn test_verification_code_generate_and_verify() {
        let manager = VerificationCodeManager::new();
        let code = manager.generate_code("13800138000");
        assert!(manager.verify_code("13800138000", &code));
        // Code should be consumed
        assert!(!manager.verify_code("13800138000", &code));
    }

    #[test]
    fn test_verification_code_wrong_code() {
        let manager = VerificationCodeManager::new();
        manager.generate_code("13800138000");
        assert!(!manager.verify_code("13800138000", "000000"));
    }

    #[test]
    fn test_verification_code_wrong_phone() {
        let manager = VerificationCodeManager::new();
        let code = manager.generate_code("13800138000");
        assert!(!manager.verify_code("13900139000", &code));
    }
}
