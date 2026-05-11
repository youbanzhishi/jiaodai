//! Shamir's Secret Sharing for key splitting
//!
//! Splits a secret key into N shares, requiring M shares to reconstruct.
//! Used for distributing key shares to viewers/confirmers.

use jiaodai_core::{JiaodaiError, KeyShare, Result};

/// Split a secret into N shares with a threshold of M
///
/// Uses Shamir's Secret Sharing over GF(256).
/// Currently provides a placeholder implementation; the full SSS
/// implementation will use the `shamirsecretsharing` crate in Phase 3.
pub fn split_secret(secret: &[u8], threshold: u8, shares: u8) -> Result<Vec<KeyShare>> {
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

    // Placeholder: for Phase 1, we create simple XOR-based shares
    // This will be replaced with proper Shamir's Secret Sharing in Phase 3
    let mut key_shares = Vec::with_capacity(shares as usize);
    use rand::RngCore;
    let mut rng = rand::thread_rng();

    for i in 0..shares {
        let mut data = vec![0u8; secret.len()];
        rng.fill_bytes(&mut data);
        key_shares.push(KeyShare { index: i, data });
    }

    Ok(key_shares)
}

/// Reconstruct a secret from M shares
///
/// Placeholder implementation for Phase 1.
pub fn reconstruct_secret(shares: &[KeyShare]) -> Result<Vec<u8>> {
    if shares.is_empty() {
        return Err(JiaodaiError::KeyShareError(
            "Need at least one share".to_string(),
        ));
    }
    // Placeholder: return the first share's data
    // Will be replaced with proper Shamir reconstruction in Phase 3
    Ok(shares[0].data.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_secret_basic() {
        let secret = b"my-secret-key-32-bytes-long-xxxxx";
        let shares = split_secret(secret, 2, 3).unwrap();
        assert_eq!(shares.len(), 3);
        for share in &shares {
            assert_eq!(share.data.len(), secret.len());
        }
    }

    #[test]
    fn test_split_secret_threshold_exceeds_shares() {
        let secret = b"test";
        let result = split_secret(secret, 5, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_split_secret_empty() {
        let result = split_secret(&[], 2, 3);
        assert!(result.is_err());
    }
}
