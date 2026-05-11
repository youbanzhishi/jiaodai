//! Application state shared across API handlers

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    // Phase 2+: database pool, seal engine, etc.
}

impl AppState {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
