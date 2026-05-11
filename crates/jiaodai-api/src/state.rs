//! Application state shared across API handlers

use crate::middleware::RateLimitConfig;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Rate limiting configuration
    pub rate_limit_config: RateLimitConfig,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            rate_limit_config: RateLimitConfig::default(),
        }
    }

    /// Create with custom rate limit config
    pub fn with_rate_limit(config: RateLimitConfig) -> Self {
        Self {
            rate_limit_config: config,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
