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

mod engine;
mod crypto;
mod shamir;
mod hash_store;
pub mod certificate;
mod event;
pub mod openlink;

pub use crypto::*;
pub use engine::*;
pub use shamir::*;
pub use hash_store::*;
pub use certificate::*;
pub use event::*;
pub use openlink::*;
