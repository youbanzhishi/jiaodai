//! # jiaodai-auth
//!
//! Account system for the Jiaodai platform.
//!
//! Responsibilities:
//! - Phone + verification code registration/login
//! - Account ↔ phone number binding (one-to-many)
//! - Phone number change and recovery flow
//! - Identity verification (OCR + liveness detection mock)
//! - JWT token management with refresh
//! - SMS service provider interface (mock)

pub mod account;
pub mod jwt;
pub mod phone;
pub mod sms;
pub mod identity;
pub mod event;

pub use account::*;
pub use jwt::*;
pub use phone::*;
pub use sms::*;
pub use identity::*;
pub use event::*;
