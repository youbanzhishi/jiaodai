//! # jiaodai-chain
//!
//! Blockchain timestamp service for the Jiaodai platform.
//!
//! Responsibilities:
//! - Merkle Tree construction for batch hash aggregation
//! - L2 contract interaction (ethers-rs)
//! - Proof verification API
//! - Local hash backup for chain-unavailable degradation
//! - Batch scheduling for periodic on-chain submission
//! - Solidity contract definition (TimestampRegistry)

pub mod merkle;
pub mod engine;
pub mod scheduler;
pub mod contract;

pub use engine::*;
pub use merkle::*;
pub use scheduler::*;
pub use contract::*;
