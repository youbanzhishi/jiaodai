//! # jiaodai-seal
//!
//! The sealing core of the Jiaodai platform.
//!
//! Responsibilities:
//! - AES-256-GCM content encryption
//! - SHA-256 content hashing
//! - Shamir's Secret Sharing key splitting
//! - SealCertificate generation

mod engine;
mod crypto;
mod shamir;

pub use crypto::*;
pub use engine::*;
pub use shamir::*;
