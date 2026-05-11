//! # jiaodai-chain
//!
//! Blockchain timestamp service for the Jiaodai platform.
//!
//! Responsibilities:
//! - Merkle Tree construction for batch hash aggregation
//! - L2 contract interaction (ethers-rs)
//! - Proof verification API
//! - Local hash backup for chain-unavailable degradation

mod merkle;
mod engine;

pub use engine::*;
pub use merkle::*;
