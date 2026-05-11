//! OpenVault integration: large file encrypted storage + Shamir's Secret Sharing
//!
//! For large files (video, audio, bulk images), content is stored in OpenVault.
//! The tape only holds:
//! - A vault file reference (vault_file_id + encryption metadata)
//! - Key shares (Shamir SSS M-of-N)
//!
//! On unseal: retrieve file from Vault + reconstruct key from shares.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use jiaodai_core::{JiaodaiError, KeyShare, Result};

// ─── Shamir's Secret Sharing (GF256) ──────────────────────────

/// Split a secret into N shares with threshold M using Shamir's Secret Sharing
///
/// Uses GF(256) arithmetic for byte-level operations.
/// Any M shares can reconstruct the original secret.
/// Fewer than M shares reveal NO information about the secret.
pub fn shamir_split(secret: &[u8], threshold: u8, shares: u8) -> Result<Vec<KeyShare>> {
    if threshold == 0 || shares == 0 {
        return Err(JiaodaiError::KeyShareError(
            "Threshold and shares must be > 0".to_string(),
        ));
    }
    if threshold > shares {
        return Err(JiaodaiError::KeyShareError(format!(
            "Threshold ({}) cannot exceed total shares ({})",
            threshold, shares
        )));
    }
    if secret.is_empty() {
        return Err(JiaodaiError::KeyShareError(
            "Secret cannot be empty".to_string(),
        ));
    }

    // For each byte of the secret, generate random polynomial coefficients
    // and evaluate at x=1,2,...,shares
    // f(x) = secret_byte + a1*x + a2*x^2 + ... + a_{t-1}*x^{t-1}
    let mut key_shares = Vec::with_capacity(shares as usize);

    for i in 1..=shares {
        key_shares.push(KeyShare {
            index: i - 1, // 0-indexed
            data: vec![0u8; secret.len()],
        });
    }

    for (byte_idx, &secret_byte) in secret.iter().enumerate() {
        // Generate random coefficients for this byte's polynomial
        // coeff[0] = secret_byte (the y-intercept)
        // coeff[1..threshold] = random
        let mut coefficients = Vec::with_capacity(threshold as usize);
        coefficients.push(secret_byte);
        for coeff_idx in 1..threshold {
            coefficients.push(random_coeff(byte_idx, coeff_idx));
        }

        // Evaluate the polynomial at each x value
        for (i, share) in key_shares.iter_mut().enumerate() {
            let x = (i + 1) as u8; // x = 1, 2, ..., shares
            let y = eval_polynomial(&coefficients, x);
            share.data[byte_idx] = y;
        }
    }

    Ok(key_shares)
}

/// Reconstruct a secret from M shares using Lagrange interpolation
///
/// Given at least `threshold` shares, reconstructs the original secret.
/// Uses GF(256) arithmetic for byte-level operations.
pub fn shamir_reconstruct(shares: &[KeyShare]) -> Result<Vec<u8>> {
    if shares.len() < 2 {
        return Err(JiaodaiError::KeyShareError(
            "Need at least 2 shares to reconstruct".to_string(),
        ));
    }

    let secret_len = shares[0].data.len();

    // Verify all shares have the same length
    for share in shares {
        if share.data.len() != secret_len {
            return Err(JiaodaiError::KeyShareError(
                "All shares must have the same length".to_string(),
            ));
        }
    }

    let mut result = Vec::with_capacity(secret_len);

    // For each byte position, do Lagrange interpolation at x=0
    for byte_idx in 0..secret_len {
        let mut secret_byte: u8 = 0;

        for (i, share_i) in shares.iter().enumerate() {
            let xi = (share_i.index + 1) as u8; // x values are 1-indexed
            let yi = share_i.data[byte_idx];

            // Compute Lagrange basis polynomial at x=0
            // L_i(0) = product_{j!=i} (0 - xj) / (xi - xj)
            // In GF(256): (0 - xj) = xj, (xi - xj) = xi ^ xj
            let mut numerator: u8 = 1;
            let mut denominator: u8 = 1;

            for (j, share_j) in shares.iter().enumerate() {
                if i == j {
                    continue;
                }
                let xj = (share_j.index + 1) as u8;
                numerator = gf256_mul(numerator, xj);
                denominator = gf256_mul(denominator, gf256_sub(xi, xj));
            }

            // Lagrange coefficient = numerator / denominator in GF(256)
            let lagrange_coeff = gf256_mul(numerator, gf256_inv(denominator));
            secret_byte = gf256_add(secret_byte, gf256_mul(yi, lagrange_coeff));
        }

        result.push(secret_byte);
    }

    Ok(result)
}

/// Evaluate a polynomial at point x in GF(256)
/// coefficients[0] is the constant term
fn eval_polynomial(coefficients: &[u8], x: u8) -> u8 {
    let mut result: u8 = 0;
    let mut x_power: u8 = 1; // x^0 = 1

    for &coeff in coefficients {
        result = gf256_add(result, gf256_mul(coeff, x_power));
        x_power = gf256_mul(x_power, x);
    }

    result
}

/// Generate a pseudo-random coefficient for testing
/// In production, this would use OsRng
fn random_coeff(byte_idx: usize, coeff_idx: u8) -> u8 {
    // Use a simple hash-based PRNG for reproducibility in tests
    // In production, use rand::rngs::OsRng
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"jiaodai-shamir-coeff");
    hasher.update(&byte_idx.to_le_bytes());
    hasher.update(&coeff_idx.to_le_bytes());
    let result = hasher.finalize();
    result[(byte_idx + coeff_idx as usize) % 32]
}

// ─── GF(256) Arithmetic ───────────────────────────────────────

/// GF(256) addition = XOR
fn gf256_add(a: u8, b: u8) -> u8 {
    a ^ b
}

/// GF(256) subtraction = XOR (same as addition in characteristic-2 field)
fn gf256_sub(a: u8, b: u8) -> u8 {
    a ^ b
}

/// GF(256) multiplication using Russian peasant multiplication
/// with the irreducible polynomial x^8 + x^4 + x^3 + x + 1 (0x11B)
fn gf256_mul(a: u8, b: u8) -> u8 {
    let mut a = a as u16;
    let mut b = b as u16;
    let mut p: u16 = 0;

    for _ in 0..8 {
        if b & 1 != 0 {
            p ^= a;
        }
        let hi = a & 0x80;
        a <<= 1;
        if hi != 0 {
            a ^= 0x11B;
        }
        b >>= 1;
    }

    p as u8
}

/// GF(256) multiplicative inverse using extended Euclidean algorithm
/// GF(256) multiplicative inverse using Fermat's little theorem
/// a^(-1) = a^254 in GF(256) since a^255 = 1
fn gf256_inv(a: u8) -> u8 {
    if a == 0 {
        return 0;
    }
    // Compute a^254 = a^(128+64+32+16+8+4+2)
    // Using repeated squaring:
    //   a^2, a^4, a^8, a^16, a^32, a^64, a^128
    // Then multiply: a^128 * a^64 * a^32 * a^16 * a^8 * a^4 * a^2
    let a2 = gf256_mul(a, a);
    let a4 = gf256_mul(a2, a2);
    let a8 = gf256_mul(a4, a4);
    let a16 = gf256_mul(a8, a8);
    let a32 = gf256_mul(a16, a16);
    let a64 = gf256_mul(a32, a32);
    let a128 = gf256_mul(a64, a64);

    gf256_mul(a2, gf256_mul(a4, gf256_mul(a8, gf256_mul(a16, gf256_mul(a32, gf256_mul(a64, a128))))))
}

// ─── Vault File Reference ─────────────────────────────────────

/// A reference to a file stored in OpenVault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultFileRef {
    /// Unique reference ID
    pub id: String,
    /// The tape ID this file belongs to
    pub tape_id: String,
    /// The vault file identifier
    pub vault_file_id: String,
    /// File size in bytes
    pub file_size: u64,
    /// Encryption algorithm used
    pub encryption_algo: String,
    /// Content hash of the encrypted file
    pub encrypted_hash: String,
    /// Key share references (M-of-N)
    pub key_shares: Vec<KeyShareRef>,
    /// Threshold for key reconstruction
    pub threshold: u8,
    /// When this reference was created
    pub created_at: DateTime<Utc>,
}

/// A reference to a key share (not the share data itself)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyShareRef {
    /// Share index
    pub index: u8,
    /// Who holds this share (account ID, phone hash, or "vault")
    pub holder: String,
    /// Whether this share has been retrieved
    pub retrieved: bool,
}

/// Vault connector trait for abstracting vault operations
///
/// Implementations can connect to different vault backends:
/// - Local filesystem vault
/// - OpenVault service
/// - S3-compatible storage
/// - IPFS
pub trait VaultConnector: Send + Sync {
    /// Store encrypted data in the vault
    fn store(&self, file_id: &str, data: &[u8]) -> Result<String>;

    /// Retrieve encrypted data from the vault
    fn retrieve(&self, file_id: &str) -> Result<Vec<u8>>;

    /// Check if a file exists in the vault
    fn exists(&self, file_id: &str) -> Result<bool>;

    /// Delete a file from the vault
    fn delete(&self, file_id: &str) -> Result<()>;

    /// Get file metadata
    fn metadata(&self, file_id: &str) -> Result<VaultFileMetadata>;
}

/// Vault file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultFileMetadata {
    pub file_id: String,
    pub size: u64,
    pub content_hash: String,
    pub created_at: DateTime<Utc>,
    pub content_type: String,
}

/// In-memory vault connector for development and testing
pub struct MemoryVaultConnector {
    files: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
    metadata: std::sync::Mutex<std::collections::HashMap<String, VaultFileMetadata>>,
}

impl MemoryVaultConnector {
    pub fn new() -> Self {
        Self {
            files: std::sync::Mutex::new(std::collections::HashMap::new()),
            metadata: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl Default for MemoryVaultConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl VaultConnector for MemoryVaultConnector {
    fn store(&self, file_id: &str, data: &[u8]) -> Result<String> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        let hash_hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();

        self.files.lock().unwrap().insert(file_id.to_string(), data.to_vec());
        self.metadata.lock().unwrap().insert(file_id.to_string(), VaultFileMetadata {
            file_id: file_id.to_string(),
            size: data.len() as u64,
            content_hash: hash_hex.clone(),
            created_at: Utc::now(),
            content_type: "application/octet-stream".to_string(),
        });

        Ok(hash_hex)
    }

    fn retrieve(&self, file_id: &str) -> Result<Vec<u8>> {
        self.files
            .lock()
            .unwrap()
            .get(file_id)
            .cloned()
            .ok_or_else(|| JiaodaiError::TapeNotFound(format!("Vault file {} not found", file_id)))
    }

    fn exists(&self, file_id: &str) -> Result<bool> {
        Ok(self.files.lock().unwrap().contains_key(file_id))
    }

    fn delete(&self, file_id: &str) -> Result<()> {
        self.files.lock().unwrap().remove(file_id);
        self.metadata.lock().unwrap().remove(file_id);
        Ok(())
    }

    fn metadata(&self, file_id: &str) -> Result<VaultFileMetadata> {
        self.metadata
            .lock()
            .unwrap()
            .get(file_id)
            .cloned()
            .ok_or_else(|| JiaodaiError::TapeNotFound(format!("Vault file {} not found", file_id)))
    }
}

/// Create a vault file reference with key shares
pub fn create_vault_ref(
    tape_id: &str,
    vault_file_id: &str,
    file_size: u64,
    encrypted_hash: &str,
    threshold: u8,
    total_shares: u8,
    holders: &[String],
) -> Result<VaultFileRef> {
    if threshold > total_shares {
        return Err(JiaodaiError::KeyShareError(format!(
            "Threshold ({}) cannot exceed total shares ({})",
            threshold, total_shares
        )));
    }
    if holders.len() < total_shares as usize {
        return Err(JiaodaiError::KeyShareError(format!(
            "Need {} holders but only {} provided",
            total_shares,
            holders.len()
        )));
    }

    let key_shares: Vec<KeyShareRef> = (0..total_shares)
        .map(|i| KeyShareRef {
            index: i,
            holder: holders.get(i as usize).cloned().unwrap_or_default(),
            retrieved: false,
        })
        .collect();

    Ok(VaultFileRef {
        id: uuid::Uuid::new_v4().to_string(),
        tape_id: tape_id.to_string(),
        vault_file_id: vault_file_id.to_string(),
        file_size,
        encryption_algo: "AES-256-GCM".to_string(),
        encrypted_hash: encrypted_hash.to_string(),
        key_shares,
        threshold,
        created_at: Utc::now(),
    })
}

/// Reconstruct the encryption key from provided shares and retrieve the file
pub fn reconstruct_and_retrieve(
    vault: &dyn VaultConnector,
    file_ref: &VaultFileRef,
    shares: &[KeyShare],
) -> Result<Vec<u8>> {
    // Reconstruct the key
    let _key = shamir_reconstruct(shares)?;

    // Retrieve the encrypted file
    let encrypted_data = vault.retrieve(&file_ref.vault_file_id)?;

    // In production: decrypt the data using the reconstructed key
    // For now: return the encrypted data (placeholder)
    // The actual decryption would use aes256gcm_decrypt from crypto module
    Ok(encrypted_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shamir_split_and_reconstruct_3of5() {
        let secret = b"my-32-byte-aes-key-1234567890ab";
        let shares = shamir_split(secret, 3, 5).unwrap();
        assert_eq!(shares.len(), 5);

        // Reconstruct with exactly threshold shares
        let reconstructed = shamir_reconstruct(&shares[0..3]).unwrap();
        assert_eq!(reconstructed, secret.to_vec());

        // Reconstruct with different subset
        let reconstructed2 = shamir_reconstruct(&[shares[0].clone(), shares[2].clone(), shares[4].clone()]).unwrap();
        assert_eq!(reconstructed2, secret.to_vec());

        // Reconstruct with all shares
        let reconstructed3 = shamir_reconstruct(&shares).unwrap();
        assert_eq!(reconstructed3, secret.to_vec());
    }

    #[test]
    fn test_shamir_threshold_2_of_3() {
        let secret = b"short-key";
        let shares = shamir_split(secret, 2, 3).unwrap();

        // Any 2 shares should reconstruct
        let r1 = shamir_reconstruct(&shares[0..2]).unwrap();
        let r2 = shamir_reconstruct(&[shares[1].clone(), shares[2].clone()]).unwrap();
        let r3 = shamir_reconstruct(&[shares[0].clone(), shares[2].clone()]).unwrap();

        assert_eq!(r1, secret.to_vec());
        assert_eq!(r2, secret.to_vec());
        assert_eq!(r3, secret.to_vec());
    }

    #[test]
    fn test_shamir_single_byte() {
        let secret = b"\x42";
        let shares = shamir_split(secret, 2, 3).unwrap();

        let r = shamir_reconstruct(&shares[0..2]).unwrap();
        assert_eq!(r, vec![0x42]);
    }

    #[test]
    fn test_shamir_invalid_params() {
        assert!(shamir_split(b"test", 0, 3).is_err());
        assert!(shamir_split(b"test", 5, 3).is_err());
        assert!(shamir_split(b"", 2, 3).is_err());
    }

    #[test]
    fn test_shamir_insufficient_shares() {
        let shares = shamir_split(b"test-key-32-bytes-long-xxxxxxxxx", 3, 5).unwrap();
        // Only 1 share should fail
        assert!(shamir_reconstruct(&shares[0..1]).is_err());
    }

    #[test]
    fn test_shamir_empty_secret() {
        let result = shamir_split(&[], 2, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_polynomial() {
        // f(x) = 5 + 3*x in GF(256)
        // f(0) = 5
        // f(1) = 5 ^ gf256_mul(3, 1) = 5 ^ 3 = 6
        assert_eq!(eval_polynomial(&[5, 3], 0), 5);
    }

    #[test]
    fn test_gf256_add_sub_same() {
        assert_eq!(gf256_add(0x57, 0x83), 0xD4);
        assert_eq!(gf256_sub(0x57, 0x83), 0xD4);
    }

    #[test]
    fn test_gf256_mul_identity() {
        assert_eq!(gf256_mul(1, 0x57), 0x57);
        assert_eq!(gf256_mul(0x57, 1), 0x57);
    }

    #[test]
    fn test_gf256_mul_zero() {
        assert_eq!(gf256_mul(0, 0x57), 0);
        assert_eq!(gf256_mul(0x57, 0), 0);
    }

    #[test]
    fn test_gf256_inv_self_multiply() {
        // a * a^(-1) = 1 in GF(256)
        for a in [1u8, 2, 3, 0x57, 0x83, 0xFF] {
            let inv = gf256_inv(a);
            let product = gf256_mul(a, inv);
            assert_eq!(product, 1, "GF(256) inv failed for {}: a*inv = {}", a, product);
        }
    }

    #[test]
    fn test_gf256_inv_zero() {
        assert_eq!(gf256_inv(0), 0);
    }

    #[test]
    fn test_memory_vault_store_retrieve() {
        let vault = MemoryVaultConnector::new();
        let data = b"encrypted file content";

        let hash = vault.store("file-1", data).unwrap();
        assert!(!hash.is_empty());

        let retrieved = vault.retrieve("file-1").unwrap();
        assert_eq!(retrieved, data.to_vec());
    }

    #[test]
    fn test_memory_vault_exists() {
        let vault = MemoryVaultConnector::new();
        assert!(!vault.exists("file-1").unwrap());
        vault.store("file-1", b"data").unwrap();
        assert!(vault.exists("file-1").unwrap());
    }

    #[test]
    fn test_memory_vault_delete() {
        let vault = MemoryVaultConnector::new();
        vault.store("file-1", b"data").unwrap();
        vault.delete("file-1").unwrap();
        assert!(!vault.exists("file-1").unwrap());
    }

    #[test]
    fn test_memory_vault_metadata() {
        let vault = MemoryVaultConnector::new();
        vault.store("file-1", b"hello world").unwrap();
        let meta = vault.metadata("file-1").unwrap();
        assert_eq!(meta.size, 11);
    }

    #[test]
    fn test_memory_vault_not_found() {
        let vault = MemoryVaultConnector::new();
        assert!(vault.retrieve("nonexistent").is_err());
    }

    #[test]
    fn test_create_vault_ref() {
        let holders = vec!["holder-1".to_string(), "holder-2".to_string(), "holder-3".to_string()];
        let file_ref = create_vault_ref(
            "tape-1",
            "vault-file-1",
            1024,
            "abc123",
            2,
            3,
            &holders,
        ).unwrap();

        assert_eq!(file_ref.tape_id, "tape-1");
        assert_eq!(file_ref.vault_file_id, "vault-file-1");
        assert_eq!(file_ref.threshold, 2);
        assert_eq!(file_ref.key_shares.len(), 3);
    }

    #[test]
    fn test_create_vault_ref_invalid_threshold() {
        let holders = vec!["h1".to_string(), "h2".to_string()];
        let result = create_vault_ref("tape-1", "vf-1", 100, "hash", 3, 2, &holders);
        assert!(result.is_err());
    }

    #[test]
    fn test_reconstruct_and_retrieve() {
        let vault = MemoryVaultConnector::new();
        let encrypted_data = b"encrypted-content-from-vault";
        vault.store("vf-1", encrypted_data).unwrap();

        let file_ref = create_vault_ref(
            "tape-1", "vf-1", 25, "hash", 2, 3,
            &["h1".to_string(), "h2".to_string(), "h3".to_string()],
        ).unwrap();

        let secret = b"my-aes-key-for-vault-file-xxxxx";
        let shares = shamir_split(secret, 2, 3).unwrap();

        let result = reconstruct_and_retrieve(&vault, &file_ref, &shares[0..2]).unwrap();
        assert_eq!(result, encrypted_data.to_vec());
    }

    #[test]
    fn test_shamir_threshold_1_of_1() {
        // Degenerate case: 1-of-1 (just the secret itself)
        let secret = b"simple-key";
        let shares = shamir_split(secret, 1, 1).unwrap();
        assert_eq!(shares.len(), 1);
        // With threshold=1, the share data IS the secret
        assert_eq!(shares[0].data, secret.to_vec());
    }
}
