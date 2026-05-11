//! JWT token management
//!
//! Provides JWT token creation and verification using HMAC-SHA256.
//! Access tokens are short-lived (1 hour), refresh tokens are long-lived (30 days).

use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use jiaodai_core::JiaodaiError;

type HmacSha256 = Hmac<Sha256>;

/// JWT claims embedded in the token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    /// Subject (account ID)
    pub sub: String,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiration (Unix timestamp)
    pub exp: i64,
    /// Token type: "access" or "refresh"
    pub token_type: String,
    /// JWT ID (unique token identifier for revocation)
    pub jti: String,
}

/// A pair of access and refresh tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// Short-lived access token
    pub access_token: String,
    /// Long-lived refresh token
    pub refresh_token: String,
    /// Access token expiration in seconds
    pub expires_in: i64,
}

/// JWT configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Secret key for HMAC-SHA256 signing
    pub secret: String,
    /// Access token duration in seconds (default: 3600)
    pub access_duration_secs: i64,
    /// Refresh token duration in seconds (default: 2592000)
    pub refresh_duration_secs: i64,
    /// Issuer name
    pub issuer: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "jiaodai-dev-secret-change-in-production".to_string(),
            access_duration_secs: 3600,     // 1 hour
            refresh_duration_secs: 2592000, // 30 days
            issuer: "jiaodai".to_string(),
        }
    }
}

/// JWT token manager
pub struct JwtManager {
    config: JwtConfig,
}

impl JwtManager {
    /// Create a new JWT manager with the given config
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    /// Create a new JWT manager with default config
    pub fn with_default() -> Self {
        Self::new(JwtConfig::default())
    }

    /// Generate a token pair (access + refresh) for the given account
    pub fn generate_token_pair(&self, account_id: &str) -> TokenPair {
        let access_token =
            self.generate_token(account_id, "access", self.config.access_duration_secs);
        let refresh_token =
            self.generate_token(account_id, "refresh", self.config.refresh_duration_secs);

        TokenPair {
            access_token,
            refresh_token,
            expires_in: self.config.access_duration_secs,
        }
    }

    /// Generate a single token
    fn generate_token(&self, account_id: &str, token_type: &str, duration_secs: i64) -> String {
        let now = Utc::now();
        let claims = TokenClaims {
            sub: account_id.to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(duration_secs)).timestamp(),
            token_type: token_type.to_string(),
            jti: uuid::Uuid::new_v4().to_string(),
        };

        let header_b64 = base64_url_encode(r#"{"alg":"HS256","typ":"JWT"}"#.as_bytes());
        let payload_str = serde_json::to_string(&claims).unwrap();
        let payload_b64 = base64_url_encode(payload_str.as_bytes());
        let signing_input = format!("{}.{}", header_b64, payload_b64);
        let signature = self.sign(&signing_input);

        format!("{}.{}", signing_input, signature)
    }

    /// Verify and decode an access token
    pub fn verify_access_token(&self, token: &str) -> Result<TokenClaims, JiaodaiError> {
        self.verify_token(token, "access")
    }

    /// Verify and decode a refresh token
    pub fn verify_refresh_token(&self, token: &str) -> Result<TokenClaims, JiaodaiError> {
        self.verify_token(token, "refresh")
    }

    /// Verify a token and check its type
    fn verify_token(&self, token: &str, expected_type: &str) -> Result<TokenClaims, JiaodaiError> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(JiaodaiError::SerializationError(
                "Invalid JWT format".to_string(),
            ));
        }

        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let expected_sig = self.sign(&signing_input);
        if parts[2] != expected_sig {
            return Err(JiaodaiError::SerializationError(
                "Invalid JWT signature".to_string(),
            ));
        }

        let payload_bytes = base64_url_decode(parts[1])
            .map_err(|e| JiaodaiError::SerializationError(format!("Invalid JWT payload: {}", e)))?;
        let payload_str = String::from_utf8(payload_bytes)
            .map_err(|e| JiaodaiError::SerializationError(format!("Invalid UTF-8: {}", e)))?;
        let claims: TokenClaims = serde_json::from_str(&payload_str)
            .map_err(|e| JiaodaiError::SerializationError(format!("Invalid JWT claims: {}", e)))?;

        if claims.token_type != expected_type {
            return Err(JiaodaiError::SerializationError(format!(
                "Expected {} token, got {}",
                expected_type, claims.token_type
            )));
        }

        let now = Utc::now().timestamp();
        if claims.exp < now {
            return Err(JiaodaiError::SerializationError(
                "Token expired".to_string(),
            ));
        }

        Ok(claims)
    }

    /// Refresh tokens using a valid refresh token
    pub fn refresh(&self, refresh_token: &str) -> Result<TokenPair, JiaodaiError> {
        let claims = self.verify_refresh_token(refresh_token)?;
        Ok(self.generate_token_pair(&claims.sub))
    }

    /// Sign data with HMAC-SHA256
    fn sign(&self, data: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.config.secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(data.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        base64_url_encode(&code_bytes)
    }
}

/// URL-safe Base64 encoding (no padding)
fn base64_url_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = if i + 1 < data.len() {
            data[i + 1] as u32
        } else {
            0
        };
        let b2 = if i + 2 < data.len() {
            data[i + 2] as u32
        } else {
            0
        };

        result.push(CHARSET[((b0 >> 2) & 0x3F) as usize] as char);
        result.push(CHARSET[(((b0 << 4) | (b1 >> 4)) & 0x3F) as usize] as char);

        if i + 1 < data.len() {
            result.push(CHARSET[(((b1 << 2) | (b2 >> 6)) & 0x3F) as usize] as char);
        }
        if i + 2 < data.len() {
            result.push(CHARSET[(b2 & 0x3F) as usize] as char);
        }

        i += 3;
    }
    result
}

/// URL-safe Base64 decoding
fn base64_url_decode(input: &str) -> Result<Vec<u8>, String> {
    const CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let input = input.trim_end_matches('=');
    let mut bytes = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits = 0;

    for c in input.chars() {
        let val = CHARSET
            .find(c)
            .ok_or_else(|| format!("Invalid base64 character: {}", c))?;
        buffer = (buffer << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_access_token() {
        let manager = JwtManager::with_default();
        let pair = manager.generate_token_pair("account-123");
        let claims = manager.verify_access_token(&pair.access_token).unwrap();
        assert_eq!(claims.sub, "account-123");
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_generate_and_verify_refresh_token() {
        let manager = JwtManager::with_default();
        let pair = manager.generate_token_pair("account-123");
        let claims = manager.verify_refresh_token(&pair.refresh_token).unwrap();
        assert_eq!(claims.sub, "account-123");
        assert_eq!(claims.token_type, "refresh");
    }

    #[test]
    fn test_access_token_cannot_be_used_as_refresh() {
        let manager = JwtManager::with_default();
        let pair = manager.generate_token_pair("account-123");
        let result = manager.verify_refresh_token(&pair.access_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_token_cannot_be_used_as_access() {
        let manager = JwtManager::with_default();
        let pair = manager.generate_token_pair("account-123");
        let result = manager.verify_access_token(&pair.refresh_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_flow() {
        let manager = JwtManager::with_default();
        let pair = manager.generate_token_pair("account-123");
        let new_pair = manager.refresh(&pair.refresh_token).unwrap();
        let claims = manager.verify_access_token(&new_pair.access_token).unwrap();
        assert_eq!(claims.sub, "account-123");
    }

    #[test]
    fn test_invalid_token_format() {
        let manager = JwtManager::with_default();
        let result = manager.verify_access_token("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_signature() {
        let manager = JwtManager::with_default();
        let pair = manager.generate_token_pair("account-123");
        let parts: Vec<&str> = pair.access_token.split('.').collect();
        let tampered = format!("{}.{}.tampered", parts[0], parts[1]);
        let result = manager.verify_access_token(&tampered);
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token() {
        let mut config = JwtConfig::default();
        config.access_duration_secs = -1; // Already expired
        let manager = JwtManager::new(config);
        let pair = manager.generate_token_pair("account-123");
        let result = manager.verify_access_token(&pair.access_token);
        assert!(result.is_err());
    }
}
