//! Cryptographic operations: AES-256-GCM encryption and SHA-256 hashing

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, AeadCore, Nonce};
use sha2::{Digest, Sha256};

use jiaodai_core::{EncryptedContent, JiaodaiError, Result};

/// Compute SHA-256 hash of data
pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Encrypt data with AES-256-GCM
pub fn aes256gcm_encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<EncryptedContent> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| JiaodaiError::EncryptionError(format!("Invalid key: {}", e)))?;

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| JiaodaiError::EncryptionError(format!("Encryption failed: {}", e)))?;

    // AES-256-GCM appends the 16-byte tag to the ciphertext
    let tag_offset = ciphertext.len() - 16;
    let mut tag = [0u8; 16];
    tag.copy_from_slice(&ciphertext[tag_offset..]);

    Ok(EncryptedContent {
        ciphertext: ciphertext[..tag_offset].to_vec(),
        nonce: nonce.into(),
        tag,
    })
}

/// Decrypt data with AES-256-GCM
pub fn aes256gcm_decrypt(encrypted: &EncryptedContent, key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| JiaodaiError::DecryptionError(format!("Invalid key: {}", e)))?;

    let nonce = Nonce::from_slice(&encrypted.nonce);
    let mut combined = encrypted.ciphertext.clone();
    combined.extend_from_slice(&encrypted.tag);

    cipher
        .decrypt(nonce, combined.as_ref())
        .map_err(|e| JiaodaiError::DecryptionError(format!("Decryption failed: {}", e)))
}

/// Generate a random 32-byte AES key
pub fn generate_aes_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    use rand::RngCore;
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hash_consistency() {
        let data = b"Hello, Jiaodai!";
        let hash1 = sha256_hash(data);
        let hash2 = sha256_hash(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_aes256gcm_roundtrip() {
        let key = generate_aes_key();
        let plaintext = b"This is a secret message for the time capsule";
        let encrypted = aes256gcm_encrypt(plaintext, &key).unwrap();
        let decrypted = aes256gcm_decrypt(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes256gcm_wrong_key_fails() {
        let key1 = generate_aes_key();
        let key2 = generate_aes_key();
        let plaintext = b"Secret content";
        let encrypted = aes256gcm_encrypt(plaintext, &key1).unwrap();
        assert!(aes256gcm_decrypt(&encrypted, &key2).is_err());
    }
}
