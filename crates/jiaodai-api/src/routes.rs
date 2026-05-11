//! API route definitions
//!
//! Reference: Blueprint Chapter 10 (API Design)

use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::state::AppState;

/// Build the complete API router
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/seal", post(seal))
        .route("/api/v1/unseal/{id}", post(unseal))
        .route("/api/v1/tape/{id}/status", get(tape_status))
        .route("/api/v1/tape/{id}/verify", get(verify_tape))
        .route("/api/v1/account/register", post(register))
        .route("/api/v1/account/login", post(login))
        .route("/api/v1/heartbeat/confirm", post(heartbeat_confirm))
        .route("/api/v1/match/check", get(match_check))
        .route("/.well-known/agent.json", get(agent_discovery))
        .with_state(state)
}

// ─── Handlers ────────────────────────────────────────────────────

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "jiaodai",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SealRequest {
    content_type: String,
    trigger_condition: Value,
    viewers: Vec<Value>,
}

async fn seal(Json(_req): Json<SealRequest>) -> Json<Value> {
    Json(json!({
        "tape_id": uuid::Uuid::new_v4().to_string(),
        "status": "sealed",
        "message": "Tape sealed successfully (placeholder)"
    }))
}

async fn unseal(axum::extract::Path(id): axum::extract::Path<String>) -> Json<Value> {
    Json(json!({
        "tape_id": id,
        "status": "condition_not_met",
        "message": "Unseal not yet implemented (placeholder)"
    }))
}

async fn tape_status(axum::extract::Path(id): axum::extract::Path<String>) -> Json<Value> {
    Json(json!({
        "tape_id": id,
        "status": "sealed",
        "message": "Status check placeholder"
    }))
}

async fn verify_tape(axum::extract::Path(id): axum::extract::Path<String>) -> Json<Value> {
    Json(json!({
        "tape_id": id,
        "verified": false,
        "message": "Verification not yet implemented (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RegisterRequest {
    phone: String,
    verification_code: String,
}

async fn register(Json(_req): Json<RegisterRequest>) -> Json<Value> {
    Json(json!({
        "account_id": uuid::Uuid::new_v4().to_string(),
        "message": "Registration not yet implemented (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LoginRequest {
    phone: String,
    verification_code: String,
}

async fn login(Json(_req): Json<LoginRequest>) -> Json<Value> {
    Json(json!({
        "message": "Login not yet implemented (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HeartbeatConfirmRequest {
    tape_id: String,
}

async fn heartbeat_confirm(Json(_req): Json<HeartbeatConfirmRequest>) -> Json<Value> {
    Json(json!({
        "message": "Heartbeat confirmation not yet implemented (placeholder)"
    }))
}

async fn match_check() -> Json<Value> {
    Json(json!({
        "matched": false,
        "message": "Match check not yet implemented (placeholder)"
    }))
}

async fn agent_discovery() -> Json<Value> {
    Json(json!({
        "name": "jiaodai",
        "description": "时间封存平台 - Seal now, open when conditions met",
        "version": env!("CARGO_PKG_VERSION"),
        "capabilities": ["seal", "unseal", "verify", "match"],
        "endpoints": {
            "seal": "POST /api/v1/seal",
            "unseal": "POST /api/v1/unseal/{id}",
            "verify": "GET /api/v1/tape/{id}/verify",
            "match": "GET /api/v1/match/check",
            "status": "GET /api/v1/tape/{id}/status"
        }
    }))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = app(AppState::new());
        let response = app
            .oneshot(Request::builder().uri("/api/v1/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_agent_discovery_endpoint() {
        let app = app(AppState::new());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/agent.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
