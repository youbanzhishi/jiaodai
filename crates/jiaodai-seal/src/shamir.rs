//! Shamir's Secret Sharing for key splitting
//!
//! Delegates to the vault module's full GF(256) implementation.
//! The vault module provides both split and reconstruct with
//! proper Shamir's Secret Sharing over GF(256).

use jiaodai_core::{KeyShare, Result};

/// Split a secret into N shares with a threshold of M
///
/// Uses Shamir's Secret Sharing over GF(256).
pub fn split_secret(secret: &[u8], threshold: u8, shares: u8) -> Result<Vec<KeyShare>> {
    crate::vault::shamir_split(secret, threshold, shares)
}

/// Reconstruct a secret from M shares
///
/// Uses Lagrange interpolation over GF(256).
pub fn reconstruct_secret(shares: &[KeyShare]) -> Result<Vec<u8>> {
    crate::vault::shamir_reconstruct(shares)
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
    fn test_split_reconstruct_roundtrip() {
        let secret = b"test-key-roundtrip-32bytes!!!!!";
        let shares = split_secret(secret, 3, 5).unwrap();

        // Reconstruct with different subsets
        let r1 = reconstruct_secret(&shares[0..3]).unwrap();
        let r2 = reconstruct_secret(&[shares[1].clone(), shares[3].clone(), shares[4].clone()]).unwrap();

        assert_eq!(r1, secret.to_vec());
        assert_eq!(r2, secret.to_vec());
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
