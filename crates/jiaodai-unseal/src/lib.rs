//! # jiaodai-unseal
//!
//! The unseal engine of the Jiaodai platform.
//!
//! Responsibilities:
//! - Condition state machine (Created → Sealed → Triggering → BufferPeriod → Confirmed → Unsealed)
//! - HeartbeatChecker: confirmer-based heartbeat with buffer period
//! - DateChecker: date-based trigger
//! - MultiConfirmerChecker: M-of-N confirmation

mod engine;
mod checkers;
mod state_machine;

pub use checkers::*;
pub use engine::*;
pub use state_machine::*;
