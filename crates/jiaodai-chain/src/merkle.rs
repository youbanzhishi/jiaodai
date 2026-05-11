//! Merkle Tree construction for batch hash aggregation
//!
//! Used to aggregate multiple content hashes into a single Merkle root
//! that is submitted to the blockchain, reducing gas costs.

use sha2::{Digest, Sha256};

/// A simple Merkle tree for content hash aggregation
pub struct MerkleTree {
    leaves: Vec<[u8; 32]>,
    root: [u8; 32],
}

impl MerkleTree {
    /// Build a Merkle tree from a list of leaf hashes
    pub fn new(leaves: Vec<[u8; 32]>) -> Self {
        let root = if leaves.is_empty() {
            [0u8; 32]
        } else if leaves.len() == 1 {
            leaves[0]
        } else {
            Self::compute_root(&leaves)
        };
        Self { leaves, root }
    }

    /// Get the Merkle root
    pub fn root(&self) -> [u8; 32] {
        self.root
    }

    /// Get the number of leaves
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// Compute the Merkle root from leaves
    fn compute_root(leaves: &[[u8; 32]]) -> [u8; 32] {
        if leaves.len() == 1 {
            return leaves[0];
        }

        let mut layer: Vec<[u8; 32]> = leaves.to_vec();

        while layer.len() > 1 {
            let mut next_layer = Vec::new();
            for chunk in layer.chunks(2) {
                let combined = if chunk.len() == 2 {
                    let mut combined = Vec::with_capacity(64);
                    combined.extend_from_slice(&chunk[0]);
                    combined.extend_from_slice(&chunk[1]);
                    combined
                } else {
                    // Odd leaf: hash it with itself
                    let mut combined = Vec::with_capacity(64);
                    combined.extend_from_slice(&chunk[0]);
                    combined.extend_from_slice(&chunk[0]);
                    combined
                };
                let mut hasher = Sha256::new();
                hasher.update(&combined);
                let result = hasher.finalize();
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&result);
                next_layer.push(hash);
            }
            layer = next_layer;
        }

        layer[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hash(val: u8) -> [u8; 32] {
        let h = [val; 32];
        h
    }

    #[test]
    fn test_merkle_empty() {
        let tree = MerkleTree::new(vec![]);
        assert!(tree.is_empty());
        assert_eq!(tree.root(), [0u8; 32]);
    }

    #[test]
    fn test_merkle_single_leaf() {
        let leaf = make_hash(1);
        let tree = MerkleTree::new(vec![leaf]);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.root(), leaf);
    }

    #[test]
    fn test_merkle_two_leaves() {
        let tree = MerkleTree::new(vec![make_hash(1), make_hash(2)]);
        assert_eq!(tree.len(), 2);
        // Root should be hash(hash1 || hash2)
        assert_ne!(tree.root(), [0u8; 32]);
    }

    #[test]
    fn test_merkle_consistency() {
        let leaves = vec![make_hash(1), make_hash(2), make_hash(3)];
        let tree1 = MerkleTree::new(leaves.clone());
        let tree2 = MerkleTree::new(leaves);
        assert_eq!(tree1.root(), tree2.root());
    }
}
