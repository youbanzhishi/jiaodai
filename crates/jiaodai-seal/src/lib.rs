//! # jiaodai-seal
//!
//! The sealing core of the Jiaodai platform.
//!
//! Responsibilities:
//! - AES-256-GCM content encryption
//! - SHA-256 content hashing
//! - Content hash storage and verification
//! - Shamir's Secret Sharing key splitting
//! - SealCertificate generation and sharing
//! - Seal event bus
//! - OpenLink Identity Card + short link sharing
//! - OpenVault integration (file references + Shamir SSS)

pub mod certificate;
mod crypto;
mod engine;
mod event;
mod hash_store;
pub mod openlink;
mod shamir;
pub mod vault;

pub use certificate::*;
pub use crypto::*;
pub use engine::*;
pub use event::*;
pub use hash_store::*;
pub use openlink::*;
pub use shamir::*;
pub use vault::*;
