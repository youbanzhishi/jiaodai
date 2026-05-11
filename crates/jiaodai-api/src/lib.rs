//! # jiaodai-api
//!
//! Axum HTTP API for the Jiaodai time-seal platform.
//!
//! Phase 11: Web frontend API preparation
//! - CORS support for cross-origin requests
//! - JWT authentication middleware
//! - WebSocket real-time notifications
//! - OpenAPI/Swagger spec endpoint
//! - Rate limiting configuration

pub mod agent;
pub mod middleware;
pub mod routes;
pub mod state;

pub use agent::*;
pub use middleware::{auth_middleware, cors_layer, NotificationEvent, RateLimitConfig};
pub use routes::app;
pub use state::AppState;
