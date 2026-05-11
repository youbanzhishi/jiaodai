//! # jiaodai-unseal
//!
//! The unseal engine of the Jiaodai platform.
//!
//! Responsibilities:
//! - Condition state machine (Draft → Sealed → Triggered → Grace → Unsealed)
//! - HeartbeatChecker: confirmer-based heartbeat with buffer period
//! - DateChecker: date-based trigger
//! - MultiConfirmerChecker: M-of-N confirmation
//! - TriggerRegistry: extensible condition registration
//! - UnsealEventBus: event-driven notifications

mod engine;
mod checkers;
mod state_machine;
mod event;
mod registry;

pub use checkers::*;
pub use engine::*;
pub use event::*;
pub use registry::*;
pub use state_machine::*;
