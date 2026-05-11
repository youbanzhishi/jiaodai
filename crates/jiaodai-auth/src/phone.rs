//! Phone number management
//!
//! Handles phone number hashing, encryption, binding, and lookup.
//! Phone numbers are hashed for privacy (search by hash) and
//! encrypted for storage (decrypt when needed for SMS).

use sha2::{Digest, Sha256};

/// Compute a SHA-256 hash of a phone number for lookup/matching
///
/// The hash is used as a privacy-preserving identifier.
/// Same phone number always produces the same hash.
pub fn phone_hash(phone: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(phone.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Simple XOR-based encryption for phone numbers (placeholder)
///
/// In production, use AES-256-GCM or a KMS-backed encryption.
/// This is a placeholder that obfuscates the phone number.
pub fn phone_encrypt(phone: &str, key: &[u8]) -> String {
    let phone_bytes = phone.as_bytes();
    let encrypted: Vec<u8> = phone_bytes
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect();
    hex::encode(&encrypted)
}

/// Decrypt a phone number from XOR-encrypted hex string
pub fn phone_decrypt(encrypted_hex: &str, key: &[u8]) -> Option<String> {
    let encrypted = hex::decode(encrypted_hex).ok()?;
    let decrypted: Vec<u8> = encrypted
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect();
    String::from_utf8(decrypted).ok()
}

/// Validate a phone number format (Chinese mobile: 11 digits starting with 1)
pub fn validate_phone_format(phone: &str) -> bool {
    if phone.len() != 11 {
        return false;
    }
    phone.starts_with('1') && phone.chars().all(|c| c.is_ascii_digit())
}

/// Generate a deterministic encryption key from a master secret
///
/// In production, this would use a KMS.
pub fn derive_phone_key(master_secret: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"jiaodai-phone-encryption-key-v1");
    hasher.update(master_secret);
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_hash_consistency() {
        let h1 = phone_hash("13800138000");
        let h2 = phone_hash("13800138000");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_phone_hash_different_phones() {
        let h1 = phone_hash("13800138000");
        let h2 = phone_hash("13900139000");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_phone_encrypt_decrypt_roundtrip() {
        let key = derive_phone_key(b"test-master-secret");
        let phone = "13800138000";
        let encrypted = phone_encrypt(phone, &key);
        let decrypted = phone_decrypt(&encrypted, &key).unwrap();
        assert_eq!(decrypted, phone);
    }

    #[test]
    fn test_phone_encrypt_different_keys() {
        let key1 = derive_phone_key(b"secret-1");
        let key2 = derive_phone_key(b"secret-2");
        let phone = "13800138000";
        let enc1 = phone_encrypt(phone, &key1);
        let enc2 = phone_encrypt(phone, &key2);
        assert_ne!(enc1, enc2);
    }

    #[test]
    fn test_validate_phone_format_valid() {
        assert!(validate_phone_format("13800138000"));
        assert!(validate_phone_format("19912345678"));
    }

    #[test]
    fn test_validate_phone_format_invalid() {
        assert!(!validate_phone_format("23800138000")); // starts with 2
        assert!(!validate_phone_format("1380013800")); // 10 digits
        assert!(!validate_phone_format("138001380000")); // 12 digits
        assert!(!validate_phone_format("1380013800a")); // non-digit
    }
}
