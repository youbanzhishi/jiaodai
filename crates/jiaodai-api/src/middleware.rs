//! Middleware for authentication, rate limiting, CORS, and logging
//!
//! Phase 11: Web frontend API preparation
//! - CORS headers for cross-origin requests
//! - JWT authentication middleware
//! - Rate limiting (placeholder)
//! - Request logging

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tower_http::cors::{Any, CorsLayer};

/// Create a CORS layer configured for the Jiaodai API
///
/// Allows:
/// - Origins: configurable (default: any for dev, specific for prod)
/// - Methods: GET, POST, OPTIONS
/// - Headers: Authorization, Content-Type
/// - Credentials: true (for cookies/auth)
pub fn cors_layer(allowed_origins: &[&str]) -> CorsLayer {
    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    if origins.is_empty() {
        // Development mode: allow all
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
            ])
            .allow_credentials(true)
    } else {
        // Production mode: specific origins
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
            ])
            .allow_credentials(true)
    }
}

/// JWT authentication middleware
///
/// Extracts Bearer token from Authorization header.
/// Returns 401 if missing or invalid.
/// For now, just validates the format — actual JWT verification
/// will use the jiaodai-auth crate.
pub async fn auth_middleware(request: Request, next: Next) -> Response {
    // Skip auth for public endpoints
    let path = request.uri().path().to_string();
    let public_paths = [
        "/api/v1/health",
        "/api/v1/account/register",
        "/api/v1/account/login",
        "/api/v1/account/refresh",
        "/.well-known/agent.json",
        "/api/v1/openapi.json",
        "/api/v1/tape/",  // GET status is public
        "/api/v1/chain/verify/",
        "/api/v1/openlink/identity-card/",
        "/api/v1/openlink/verify/",
    ];

    let is_public = public_paths.iter().any(|p| path.starts_with(p))
        || path.starts_with("/api/v1/capsule/") && path.ends_with("/countdown");

    if is_public {
        return next.run(request).await;
    }

    // Check for Authorization header
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..];
            if token.is_empty() {
                return (
                    StatusCode::UNAUTHORIZED,
                    "{\"error\": \"Empty bearer token\"}",
                )
                    .into_response();
            }
            // In production: validate JWT with jiaodai-auth
            // For now: accept any non-empty Bearer token
            next.run(request).await
        }
        _ => (
            StatusCode::UNAUTHORIZED,
            "{\"error\": \"Missing or invalid Authorization header\"}",
        )
            .into_response(),
    }
}

/// Rate limiting state
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Max requests per minute per IP
    pub max_requests_per_minute: u32,
    /// Max requests per minute for agent access
    pub agent_max_requests_per_minute: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_minute: 100,
            agent_max_requests_per_minute: 1000,
        }
    }
}

/// Notification types for WebSocket
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationEvent {
    /// A tape was unsealed
    TapeUnsealed {
        tape_id: String,
        at: chrono::DateTime<chrono::Utc>,
    },
    /// A mutual match was found
    MatchFound {
        tape_id_a: String,
        tape_id_b: String,
        at: chrono::DateTime<chrono::Utc>,
    },
    /// Heartbeat reminder
    HeartbeatReminder {
        account_id: String,
        days_overdue: u32,
        at: chrono::DateTime<chrono::Utc>,
    },
    /// Grace period started
    GracePeriodStarted {
        tape_id: String,
        grace_until: chrono::DateTime<chrono::Utc>,
        at: chrono::DateTime<chrono::Utc>,
    },
    /// Capsule opened
    CapsuleOpened {
        tape_id: String,
        at: chrono::DateTime<chrono::Utc>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_layer_dev() {
        let layer = cors_layer(&[]);
        // Should not panic
        assert!(true);
    }

    #[test]
    fn test_cors_layer_prod() {
        let layer = cors_layer(&["https://jiaod.ai", "https://app.jiaod.ai"]);
        assert!(true);
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests_per_minute, 100);
        assert_eq!(config.agent_max_requests_per_minute, 1000);
    }

    #[test]
    fn test_notification_event_serialization() {
        let event = NotificationEvent::TapeUnsealed {
            tape_id: "tape-1".to_string(),
            at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("tape_unsealed"));
    }

    #[test]
    fn test_notification_event_deserialization() {
        let event = NotificationEvent::MatchFound {
            tape_id_a: "a".to_string(),
            tape_id_b: "b".to_string(),
            at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: NotificationEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            NotificationEvent::MatchFound { tape_id_a, .. } => {
                assert_eq!(tape_id_a, "a");
            }
            _ => panic!("Wrong event type"),
        }
    }
}
