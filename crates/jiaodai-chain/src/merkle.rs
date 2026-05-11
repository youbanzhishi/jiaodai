//! Merkle Tree construction for batch hash aggregation
//!
//! Used to aggregate multiple content hashes into a single Merkle root
//! that is submitted to the blockchain, reducing gas costs.
//! Also provides Merkle proof generation and verification.

use sha2::{Digest, Sha256};

/// A Merkle tree for content hash aggregation with proof support
pub struct MerkleTree {
    leaves: Vec<[u8; 32]>,
    root: [u8; 32],
    /// Pre-computed layers for proof generation
    layers: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    /// Build a Merkle tree from a list of leaf hashes
    pub fn new(leaves: Vec<[u8; 32]>) -> Self {
        if leaves.is_empty() {
            return Self {
                leaves,
                root: [0u8; 32],
                layers: vec![],
            };
        }

        let mut layers = vec![leaves.clone()];
        let mut current = leaves.clone();

        while current.len() > 1 {
            let mut next = Vec::new();
            for chunk in current.chunks(2) {
                let combined = if chunk.len() == 2 {
                    let mut c = Vec::with_capacity(64);
                    c.extend_from_slice(&chunk[0]);
                    c.extend_from_slice(&chunk[1]);
                    c
                } else {
                    // Odd leaf: hash it with itself
                    let mut c = Vec::with_capacity(64);
                    c.extend_from_slice(&chunk[0]);
                    c.extend_from_slice(&chunk[0]);
                    c
                };
                let mut hasher = Sha256::new();
                hasher.update(&combined);
                let result = hasher.finalize();
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&result);
                next.push(hash);
            }
            layers.push(next.clone());
            current = next;
        }

        let root = layers
            .last()
            .and_then(|l| l.first())
            .copied()
            .unwrap_or([0u8; 32]);

        Self {
            leaves,
            root,
            layers,
        }
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

    /// Get the leaf hashes
    pub fn leaves(&self) -> &[[u8; 32]] {
        &self.leaves
    }

    /// Generate a Merkle proof for a leaf at the given index
    ///
    /// Returns the sibling hashes needed to verify the leaf is part of the tree.
    /// Each proof element contains a sibling hash and direction indicator.
    pub fn proof(&self, leaf_index: usize) -> Option<MerkleProof> {
        if leaf_index >= self.leaves.len() || self.layers.is_empty() {
            return None;
        }

        let mut path = Vec::new();
        let mut index = leaf_index;

        for layer in &self.layers {
            if layer.len() <= 1 {
                break;
            }
            let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            if sibling_index < layer.len() {
                let is_right = index % 2 == 0;
                path.push(ProofNode {
                    hash: layer[sibling_index],
                    is_right,
                });
            } else {
                // Odd node at end: pair with itself
                path.push(ProofNode {
                    hash: layer[index],
                    is_right: true,
                });
            }
            index /= 2;
        }

        Some(MerkleProof {
            leaf: self.leaves[leaf_index],
            leaf_index,
            path,
            root: self.root,
        })
    }
}

/// A node in a Merkle proof path
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofNode {
    /// The sibling hash
    pub hash: [u8; 32],
    /// Whether the sibling is on the right side
    pub is_right: bool,
}

/// A Merkle proof for verifying a leaf is part of the tree
#[derive(Debug, Clone)]
pub struct MerkleProof {
    /// The leaf hash being proven
    pub leaf: [u8; 32],
    /// The index of the leaf in the tree
    pub leaf_index: usize,
    /// The proof path (sibling hashes)
    pub path: Vec<ProofNode>,
    /// The expected Merkle root
    pub root: [u8; 32],
}

impl MerkleProof {
    /// Verify this Merkle proof
    pub fn verify(&self) -> bool {
        let mut current = self.leaf;

        for node in &self.path {
            let mut hasher = Sha256::new();
            if node.is_right {
                hasher.update(&current);
                hasher.update(&node.hash);
            } else {
                hasher.update(&node.hash);
                hasher.update(&current);
            }
            let result = hasher.finalize();
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&result);
            current = hash;
        }

        current == self.root
    }

    /// Convert proof path to hex strings for serialization
    pub fn path_hex(&self) -> Vec<String> {
        self.path
            .iter()
            .map(|n| n.hash.iter().map(|b| format!("{:02x}", b)).collect())
            .collect()
    }

    /// Convert leaf hash to hex string
    pub fn leaf_hex(&self) -> String {
        self.leaf.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Convert root hash to hex string
    pub fn root_hex(&self) -> String {
        self.root.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

/// Verify a Merkle proof given the leaf, path, and expected root
pub fn verify_merkle_proof(leaf: &[u8; 32], path: &[ProofNode], expected_root: &[u8; 32]) -> bool {
    let mut current = *leaf;

    for node in path {
        let mut hasher = Sha256::new();
        if node.is_right {
            hasher.update(&current);
            hasher.update(&node.hash);
        } else {
            hasher.update(&node.hash);
            hasher.update(&current);
        }
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        current = hash;
    }

    current == *expected_root
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hash(val: u8) -> [u8; 32] {
        [val; 32]
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
        assert_ne!(tree.root(), [0u8; 32]);
    }

    #[test]
    fn test_merkle_consistency() {
        let leaves = vec![make_hash(1), make_hash(2), make_hash(3)];
        let tree1 = MerkleTree::new(leaves.clone());
        let tree2 = MerkleTree::new(leaves);
        assert_eq!(tree1.root(), tree2.root());
    }

    #[test]
    fn test_merkle_proof_single_leaf() {
        let leaf = make_hash(42);
        let tree = MerkleTree::new(vec![leaf]);
        let proof = tree.proof(0).unwrap();
        assert!(proof.verify());
    }

    #[test]
    fn test_merkle_proof_two_leaves() {
        let leaves = vec![make_hash(1), make_hash(2)];
        let tree = MerkleTree::new(leaves);

        let proof0 = tree.proof(0).unwrap();
        let proof1 = tree.proof(1).unwrap();

        assert!(proof0.verify());
        assert!(proof1.verify());
    }

    #[test]
    fn test_merkle_proof_four_leaves() {
        let leaves = vec![make_hash(1), make_hash(2), make_hash(3), make_hash(4)];
        let tree = MerkleTree::new(leaves);

        for i in 0..4 {
            let proof = tree.proof(i).unwrap();
            assert!(proof.verify(), "Proof for leaf {} should verify", i);
        }
    }

    #[test]
    fn test_merkle_proof_odd_leaves() {
        let leaves = vec![make_hash(1), make_hash(2), make_hash(3)];
        let tree = MerkleTree::new(leaves);

        for i in 0..3 {
            let proof = tree.proof(i).unwrap();
            assert!(proof.verify(), "Proof for leaf {} should verify", i);
        }
    }

    #[test]
    fn test_merkle_proof_out_of_bounds() {
        let tree = MerkleTree::new(vec![make_hash(1), make_hash(2)]);
        assert!(tree.proof(5).is_none());
    }

    #[test]
    fn test_verify_merkle_proof_standalone() {
        let leaves = vec![make_hash(10), make_hash(20), make_hash(30)];
        let tree = MerkleTree::new(leaves);
        let proof = tree.proof(1).unwrap();

        assert!(verify_merkle_proof(&proof.leaf, &proof.path, &proof.root));
    }

    #[test]
    fn test_proof_hex_serialization() {
        let tree = MerkleTree::new(vec![make_hash(1), make_hash(2)]);
        let proof = tree.proof(0).unwrap();
        let path_hex = proof.path_hex();
        assert!(!path_hex.is_empty());
        for hex in &path_hex {
            assert_eq!(hex.len(), 64);
        }
    }
}
