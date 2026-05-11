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

pub mod routes;
pub mod state;
pub mod middleware;

pub use routes::app;
pub use state::AppState;
pub use middleware::{cors_layer, auth_middleware, RateLimitConfig, NotificationEvent};
