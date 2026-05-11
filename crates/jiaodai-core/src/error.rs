//! Error types for jiaodai-core

use thiserror::Error;

/// Core error type for the Jiaodai platform
#[derive(Error, Debug)]
pub enum JiaodaiError {
    #[error("Encryption failed: {0}")]
    EncryptionError(String),

    #[error("Decryption failed: {0}")]
    DecryptionError(String),

    #[error("Hash verification failed: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Invalid state transition: from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Condition not met: {0}")]
    ConditionNotMet(String),

    #[error("Viewer verification failed: {0}")]
    ViewerVerificationFailed(String),

    #[error("Key share error: {0}")]
    KeyShareError(String),

    #[error("Tape not found: {0}")]
    TapeNotFound(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, JiaodaiError>;
