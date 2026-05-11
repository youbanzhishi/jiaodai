//! # jiaodai-api
//!
//! Axum HTTP API for the Jiaodai time-seal platform.

pub mod routes;
pub mod state;

pub use routes::app;
pub use state::AppState;
