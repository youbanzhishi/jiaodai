//! # jiaodai-chain
//!
//! Blockchain timestamp service for the Jiaodai platform.
//!
//! Responsibilities:
//! - Merkle Tree construction with proof generation/verification
//! - L2 contract interaction (ethers-rs)
//! - Mock chain engine for development
//! - Timestamp verification API
//! - Local hash backup for chain-unavailable degradation
//! - Batch scheduling for periodic on-chain submission (threshold + timer)
//! - Solidity contract definition (TimestampRegistry)
//!
//! Architecture: ChainTimestamp trait isolates chain implementation.
//! Swap chain by providing a different impl — business code unchanged.

pub mod merkle;
pub mod engine;
pub mod scheduler;
pub mod contract;

pub use engine::*;
pub use merkle::*;
pub use scheduler::*;
pub use contract::*;
