//! API route definitions
//!
//! Reference: Blueprint Chapter 10 (API Design)
//!
//! Covers all phases:
//! - Phase 2: Account (register/login/bind/refresh/identity)
//! - Phase 3: Seal (create seal/verify/hash)
//! - Phase 4: Unseal (check/unseal/heartbeat/match)
//! - Phase 5: Crush (search phone)
//! - Phase 6: Will (create will/heartbeat)
//! - Phase 7: Capsule (create capsule/countdown)
//! - Phase 8: Chain (batch submit/verify)
//! - Phase 9: OpenLink (identity card/short link/verify)
//! - Phase 10: OpenVault (vault ref/key shares)
//! - Phase 11: CORS + auth + WebSocket + OpenAPI
//! - Phase 12: Agent Action Protocol

use axum::{
    extract::{ws::Message, Path, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::state::AppState;

/// Build the complete API router
pub fn app(state: AppState) -> Router {
    Router::new()
        // Phase 1: Health
        .route("/api/v1/health", get(health))
        // Phase 2: Account
        .route("/api/v1/account/register", post(register))
        .route("/api/v1/account/login", post(login))
        .route("/api/v1/account/refresh", post(refresh_token))
        .route("/api/v1/account/bind-phone", post(bind_phone))
        .route("/api/v1/account/change-phone", post(change_phone))
        .route("/api/v1/account/identity-verify", post(identity_verify))
        .route("/api/v1/account/recover", post(recover_account))
        // Phase 3: Seal
        .route("/api/v1/seal", post(seal))
        .route("/api/v1/tape/{id}/status", get(tape_status))
        .route("/api/v1/tape/{id}/verify", get(verify_tape))
        .route("/api/v1/tape/{id}/certificate", get(get_certificate))
        .route("/api/v1/tape/{id}/share", post(share_certificate))
        // Phase 4: Unseal
        .route("/api/v1/unseal/{id}", post(unseal))
        .route("/api/v1/heartbeat/confirm", post(heartbeat_confirm))
        .route("/api/v1/match/check", get(match_check))
        // Phase 5: Crush
        .route("/api/v1/crush/search", post(crush_search))
        .route("/api/v1/crush/create", post(crush_create))
        // Phase 6: Will
        .route("/api/v1/will/create", post(will_create))
        .route("/api/v1/will/heartbeat", post(will_heartbeat))
        .route("/api/v1/will/list", get(will_list))
        // Phase 7: Capsule
        .route("/api/v1/capsule/create", post(capsule_create))
        .route("/api/v1/capsule/{id}/countdown", get(capsule_countdown))
        .route("/api/v1/capsule/list", get(capsule_list))
        // Phase 8: Chain
        .route("/api/v1/chain/batch", get(chain_batch_status))
        .route("/api/v1/chain/submit", post(chain_batch_submit))
        .route("/api/v1/chain/verify/{tape_id}", get(chain_verify))
        // Phase 9: OpenLink
        .route(
            "/api/v1/openlink/identity-card/{tape_id}",
            get(openlink_identity_card),
        )
        .route(
            "/api/v1/openlink/short-link/{tape_id}",
            get(openlink_short_link),
        )
        .route(
            "/api/v1/openlink/short-link/{short_code}/access",
            post(openlink_access),
        )
        .route("/api/v1/openlink/verify/{tape_id}", post(openlink_verify))
        // Phase 10: OpenVault
        .route("/api/v1/vault/ref", post(vault_create_ref))
        .route("/api/v1/vault/key-shares", post(vault_key_shares))
        .route("/api/v1/vault/retrieve/{tape_id}", post(vault_retrieve))
        // Phase 11: WebSocket for real-time notifications
        .route("/api/v1/ws/notifications", get(ws_notifications))
        // Phase 11: OpenAPI spec
        .route("/api/v1/openapi.json", get(openapi_spec))
        // Phase 12: Agent Action Protocol
        .route("/.well-known/agent.json", get(agent_discovery))
        .with_state(state)
}

// ─── Health ────────────────────────────────────────────────────

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "jiaodai",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// ─── Phase 2: Account ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SendCodeRequest {
    phone: String,
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    phone: String,
    verification_code: String,
}

async fn register(Json(_req): Json<RegisterRequest>) -> Json<Value> {
    Json(json!({
        "account_id": uuid::Uuid::new_v4().to_string(),
        "message": "Registration endpoint ready (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    phone: String,
    verification_code: String,
}

async fn login(Json(_req): Json<LoginRequest>) -> Json<Value> {
    Json(json!({
        "access_token": "jwt-access-token-placeholder",
        "refresh_token": "jwt-refresh-token-placeholder",
        "expires_in": 3600,
        "message": "Login endpoint ready (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

async fn refresh_token(Json(_req): Json<RefreshRequest>) -> Json<Value> {
    Json(json!({
        "access_token": "new-jwt-access-token",
        "refresh_token": "new-jwt-refresh-token",
        "expires_in": 3600,
        "message": "Token refresh endpoint ready (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
struct BindPhoneRequest {
    account_id: String,
    phone: String,
    verification_code: String,
}

async fn bind_phone(Json(_req): Json<BindPhoneRequest>) -> Json<Value> {
    Json(json!({
        "message": "Phone binding endpoint ready (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
struct ChangePhoneRequest {
    account_id: String,
    old_phone: String,
    old_code: String,
    new_phone: String,
    new_code: String,
}

async fn change_phone(Json(_req): Json<ChangePhoneRequest>) -> Json<Value> {
    Json(json!({
        "message": "Phone change endpoint ready (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
struct IdentityVerifyRequest {
    account_id: String,
    #[allow(dead_code)]
    id_card_image: String,
    #[allow(dead_code)]
    face_image: String,
}

async fn identity_verify(Json(_req): Json<IdentityVerifyRequest>) -> Json<Value> {
    Json(json!({
        "verified": true,
        "message": "Identity verification endpoint ready (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
struct RecoverRequest {
    id_number_hash: String,
    new_phone: String,
    new_code: String,
}

async fn recover_account(Json(_req): Json<RecoverRequest>) -> Json<Value> {
    Json(json!({
        "message": "Account recovery endpoint ready (placeholder)"
    }))
}

// ─── Phase 3: Seal ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
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

async fn tape_status(Path(id): Path<String>) -> Json<Value> {
    Json(json!({
        "tape_id": id,
        "status": "sealed",
        "message": "Status check placeholder"
    }))
}

async fn verify_tape(Path(id): Path<String>) -> Json<Value> {
    Json(json!({
        "tape_id": id,
        "verified": false,
        "message": "Verification not yet implemented (placeholder)"
    }))
}

async fn get_certificate(Path(id): Path<String>) -> Json<Value> {
    Json(json!({
        "tape_id": id,
        "certificate": {
            "content_hash": "sha256-hash-placeholder",
            "sealed_at": "2026-01-01T00:00:00Z",
            "trigger_condition": "date_trigger",
            "viewers": []
        },
        "message": "Certificate endpoint ready (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
struct ShareRequest {
    method: String,
}

async fn share_certificate(Path(id): Path<String>, Json(_req): Json<ShareRequest>) -> Json<Value> {
    Json(json!({
        "tape_id": id,
        "short_link": format!("https://jiaod.ai/s/{}", id),
        "qr_data": format!("jiaodai://tape/{}", id),
        "message": "Share endpoint ready (placeholder)"
    }))
}

// ─── Phase 4: Unseal ───────────────────────────────────────────

async fn unseal(Path(id): Path<String>) -> Json<Value> {
    Json(json!({
        "tape_id": id,
        "status": "condition_not_met",
        "message": "Unseal endpoint ready (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
struct HeartbeatConfirmRequest {
    tape_id: String,
}

async fn heartbeat_confirm(Json(_req): Json<HeartbeatConfirmRequest>) -> Json<Value> {
    Json(json!({
        "message": "Heartbeat confirmation endpoint ready (placeholder)"
    }))
}

async fn match_check() -> Json<Value> {
    Json(json!({
        "matched": false,
        "message": "Match check endpoint ready (placeholder)"
    }))
}

// ─── Phase 5: Crush ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CrushSearchRequest {
    phone: String,
}

async fn crush_search(Json(_req): Json<CrushSearchRequest>) -> Json<Value> {
    Json(json!({
        "registered": false,
        "message": "Crush search endpoint ready (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
struct CrushCreateRequest {
    creator_id: String,
    creator_phone: String,
    target_phone: String,
}

async fn crush_create(Json(_req): Json<CrushCreateRequest>) -> Json<Value> {
    Json(json!({
        "tape_id": uuid::Uuid::new_v4().to_string(),
        "invitation_sent": false,
        "matched": false,
        "message": "Crush create endpoint ready (placeholder)"
    }))
}

// ─── Phase 6: Will ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct WillCreateRequest {
    creator_id: String,
    heartbeat_interval_days: u32,
    grace_period_days: u32,
    #[allow(dead_code)]
    viewers: Vec<Value>,
}

async fn will_create(Json(_req): Json<WillCreateRequest>) -> Json<Value> {
    Json(json!({
        "tape_id": uuid::Uuid::new_v4().to_string(),
        "status": "active",
        "message": "Will creation endpoint ready (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
struct WillHeartbeatRequest {
    account_id: String,
}

async fn will_heartbeat(Json(_req): Json<WillHeartbeatRequest>) -> Json<Value> {
    Json(json!({
        "received": true,
        "message": "Will heartbeat endpoint ready (placeholder)"
    }))
}

async fn will_list() -> Json<Value> {
    Json(json!({
        "wills": [],
        "message": "Will list endpoint ready (placeholder)"
    }))
}

// ─── Phase 7: Capsule ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CapsuleCreateRequest {
    creator_id: String,
    open_at: String,
    #[allow(dead_code)]
    viewers: Vec<Value>,
    timezone: String,
}

async fn capsule_create(Json(_req): Json<CapsuleCreateRequest>) -> Json<Value> {
    Json(json!({
        "tape_id": uuid::Uuid::new_v4().to_string(),
        "status": "sealed",
        "short_link": "https://jiaod.ai/s/placeholder",
        "message": "Capsule creation endpoint ready (placeholder)"
    }))
}

async fn capsule_countdown(Path(id): Path<String>) -> Json<Value> {
    Json(json!({
        "tape_id": id,
        "days_remaining": 365,
        "message": "Capsule countdown endpoint ready (placeholder)"
    }))
}

async fn capsule_list() -> Json<Value> {
    Json(json!({
        "capsules": [],
        "message": "Capsule list endpoint ready (placeholder)"
    }))
}

// ─── Phase 8: Chain ────────────────────────────────────────────

async fn chain_batch_status() -> Json<Value> {
    Json(json!({
        "pending_batches": 0,
        "message": "Chain batch status endpoint ready (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
struct ChainBatchSubmitRequest {
    #[allow(dead_code)]
    force: Option<bool>,
}

async fn chain_batch_submit(Json(_req): Json<ChainBatchSubmitRequest>) -> Json<Value> {
    Json(json!({
        "submitted": true,
        "batch_id": uuid::Uuid::new_v4().to_string(),
        "tape_count": 0,
        "message": "Chain batch submit endpoint ready (placeholder)"
    }))
}

async fn chain_verify(Path(tape_id): Path<String>) -> Json<Value> {
    Json(json!({
        "tape_id": tape_id,
        "on_chain": false,
        "merkle_root": null,
        "tx_hash": null,
        "block_number": null,
        "timestamp": null,
        "merkle_proof": [],
        "message": "Chain verify endpoint ready (placeholder)"
    }))
}

// ─── Phase 9: OpenLink ─────────────────────────────────────────

async fn openlink_identity_card(Path(tape_id): Path<String>) -> Json<Value> {
    Json(json!({
        "type": "jiaodai-seal-certificate",
        "tape_id": tape_id,
        "content_hash": "sha256-hash-placeholder",
        "sealed_at": "2026-01-01T00:00:00Z",
        "trigger_condition_summary": "date_trigger",
        "chain_proof": null,
        "fingerprint": "abc123def456",
        "version": "1.0.0",
        "message": "OpenLink Identity Card endpoint ready (placeholder)"
    }))
}

async fn openlink_short_link(Path(tape_id): Path<String>) -> Json<Value> {
    Json(json!({
        "tape_id": tape_id,
        "short_link": format!("https://jiaod.ai/s/{}", tape_id),
        "short_code": "abc12345",
        "created_at": "2026-01-01T00:00:00Z",
        "expires_at": null,
        "message": "OpenLink short link endpoint ready (placeholder)"
    }))
}

async fn openlink_access(Path(short_code): Path<String>) -> Json<Value> {
    Json(json!({
        "short_code": short_code,
        "access_count": 1,
        "message": "Short link access recorded (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
struct OpenLinkVerifyRequest {
    #[allow(dead_code)]
    content_hash: Option<String>,
}

async fn openlink_verify(
    Path(tape_id): Path<String>,
    Json(_req): Json<OpenLinkVerifyRequest>,
) -> Json<Value> {
    Json(json!({
        "tape_id": tape_id,
        "valid": true,
        "hash_valid": true,
        "chain_verified": null,
        "timestamp_valid": true,
        "message": "OpenLink verification endpoint ready (placeholder)"
    }))
}

// ─── Phase 10: OpenVault ───────────────────────────────────────

#[derive(Debug, Deserialize)]
struct VaultRefRequest {
    tape_id: String,
    #[allow(dead_code)]
    vault_file_id: String,
    #[allow(dead_code)]
    key_shares: Vec<Value>,
}

async fn vault_create_ref(Json(_req): Json<VaultRefRequest>) -> Json<Value> {
    Json(json!({
        "ref_id": uuid::Uuid::new_v4().to_string(),
        "message": "Vault reference creation endpoint ready (placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
struct KeySharesRequest {
    tape_id: String,
    #[allow(dead_code)]
    threshold: u8,
    #[allow(dead_code)]
    total_shares: u8,
}

async fn vault_key_shares(Json(_req): Json<KeySharesRequest>) -> Json<Value> {
    Json(json!({
        "shares": [],
        "message": "Vault key shares endpoint ready (Shamir SSS placeholder)"
    }))
}

#[derive(Debug, Deserialize)]
struct VaultRetrieveRequest {
    #[allow(dead_code)]
    shares: Vec<Value>,
}

async fn vault_retrieve(
    Path(tape_id): Path<String>,
    Json(_req): Json<VaultRetrieveRequest>,
) -> Json<Value> {
    Json(json!({
        "tape_id": tape_id,
        "file_retrieved": false,
        "key_reconstructed": false,
        "message": "Vault retrieve endpoint ready (placeholder)"
    }))
}

// ─── Phase 11: WebSocket ───────────────────────────────────────

async fn ws_notifications(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|mut socket| async move {
        // Send a welcome message
        let _ = socket
            .send(Message::Text("Connected to Jiaodai notifications".into()))
            .await;
        // In production, this would subscribe to the event bus and forward events
    })
}

// ─── Phase 11: OpenAPI Spec ────────────────────────────────────

async fn openapi_spec() -> Json<Value> {
    Json(json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Jiaodai API",
            "description": "时间封存平台 - Seal now, open when conditions met",
            "version": env!("CARGO_PKG_VERSION")
        },
        "servers": [
            { "url": "https://api.jiaod.ai", "description": "Production" },
            { "url": "http://localhost:3000", "description": "Development" }
        ],
        "paths": {
            "/api/v1/health": {
                "get": { "summary": "Health check", "responses": { "200": { "description": "OK" } } }
            },
            "/api/v1/account/register": {
                "post": { "summary": "Register new account", "tags": ["Account"], "requestBody": { "content": { "application/json": { "schema": { "type": "object", "properties": { "phone": { "type": "string" }, "verification_code": { "type": "string" } }, "required": ["phone", "verification_code"] } } } }, "responses": { "200": { "description": "Account created" } } }
            },
            "/api/v1/account/login": {
                "post": { "summary": "Login", "tags": ["Account"], "requestBody": { "content": { "application/json": { "schema": { "type": "object", "properties": { "phone": { "type": "string" }, "verification_code": { "type": "string" } }, "required": ["phone", "verification_code"] } } } }, "responses": { "200": { "description": "JWT tokens" } } }
            },
            "/api/v1/seal": {
                "post": { "summary": "Create sealed tape", "tags": ["Seal"], "security": [{ "bearerAuth": [] }], "requestBody": { "content": { "application/json": { "schema": { "type": "object", "properties": { "content_type": { "type": "string", "enum": ["text", "image", "video", "file", "vault_ref"] }, "trigger_condition": { "type": "object" }, "viewers": { "type": "array", "items": { "type": "object" } } }, "required": ["content_type", "trigger_condition", "viewers"] } } } }, "responses": { "200": { "description": "Tape sealed" } } }
            },
            "/api/v1/tape/{id}/status": {
                "get": { "summary": "Get tape status", "tags": ["Tape"], "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string" } }], "responses": { "200": { "description": "Tape status" } } }
            },
            "/api/v1/unseal/{id}": {
                "post": { "summary": "Attempt unseal", "tags": ["Unseal"], "security": [{ "bearerAuth": [] }], "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string" } }], "responses": { "200": { "description": "Unseal result" } } }
            },
            "/api/v1/chain/verify/{tape_id}": {
                "get": { "summary": "Verify on-chain timestamp", "tags": ["Chain"], "parameters": [{ "name": "tape_id", "in": "path", "required": true, "schema": { "type": "string" } }], "responses": { "200": { "description": "Timestamp verification" } } }
            },
            "/api/v1/openlink/identity-card/{tape_id}": {
                "get": { "summary": "Get Identity Card for tape", "tags": ["OpenLink"], "parameters": [{ "name": "tape_id", "in": "path", "required": true, "schema": { "type": "string" } }], "responses": { "200": { "description": "Identity Card" } } }
            },
            "/api/v1/openlink/verify/{tape_id}": {
                "post": { "summary": "Verify Identity Card", "tags": ["OpenLink"], "parameters": [{ "name": "tape_id", "in": "path", "required": true, "schema": { "type": "string" } }], "responses": { "200": { "description": "Verification result" } } }
            },
            "/api/v1/vault/ref": {
                "post": { "summary": "Create vault file reference", "tags": ["Vault"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "Vault reference created" } } }
            },
            "/api/v1/vault/retrieve/{tape_id}": {
                "post": { "summary": "Retrieve vault file with key reconstruction", "tags": ["Vault"], "security": [{ "bearerAuth": [] }], "parameters": [{ "name": "tape_id", "in": "path", "required": true, "schema": { "type": "string" } }], "responses": { "200": { "description": "File retrieved" } } }
            },
            "/.well-known/agent.json": {
                "get": { "summary": "Agent discovery", "tags": ["Agent"], "responses": { "200": { "description": "Agent capabilities" } } }
            }
        },
        "components": {
            "securitySchemes": {
                "bearerAuth": { "type": "http", "scheme": "bearer", "bearerFormat": "JWT" }
            }
        },
        "tags": [
            { "name": "Account", "description": "User account management" },
            { "name": "Seal", "description": "Create and manage sealed tapes" },
            { "name": "Tape", "description": "Tape status and verification" },
            { "name": "Unseal", "description": "Unseal operations" },
            { "name": "Chain", "description": "Blockchain timestamp operations" },
            { "name": "OpenLink", "description": "Identity Card and sharing" },
            { "name": "Vault", "description": "Encrypted file storage" },
            { "name": "Agent", "description": "Agent Action Protocol" }
        ]
    }))
}

// ─── Phase 12: Agent Action Protocol ───────────────────────────

async fn agent_discovery() -> Json<Value> {
    Json(json!({
        "name": "jiaodai",
        "description": "时间封存平台 - Seal now, open when conditions met",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol_version": "1.0.0",
        "capabilities": {
            "seal": {
                "description": "Create a sealed tape with encrypted content",
                "endpoint": "POST /api/v1/seal",
                "parameters": ["content_type", "trigger_condition", "viewers"],
                "auth_required": true
            },
            "unseal": {
                "description": "Attempt to unseal a tape",
                "endpoint": "POST /api/v1/unseal/{id}",
                "parameters": ["identity_claim"],
                "auth_required": true
            },
            "verify": {
                "description": "Verify a tape's integrity and chain proof",
                "endpoint": "GET /api/v1/tape/{id}/verify",
                "parameters": ["tape_id"],
                "auth_required": false
            },
            "status": {
                "description": "Get tape status",
                "endpoint": "GET /api/v1/tape/{id}/status",
                "parameters": ["tape_id"],
                "auth_required": true
            },
            "match": {
                "description": "Check for mutual match",
                "endpoint": "GET /api/v1/match/check",
                "parameters": ["tape_id"],
                "auth_required": true
            },
            "crush_search": {
                "description": "Search phone number for crush scenario",
                "endpoint": "POST /api/v1/crush/search",
                "parameters": ["phone"],
                "auth_required": true
            },
            "crush_create": {
                "description": "Create crush seal",
                "endpoint": "POST /api/v1/crush/create",
                "parameters": ["creator_id", "creator_phone", "target_phone"],
                "auth_required": true
            },
            "will_create": {
                "description": "Create will with heartbeat trigger",
                "endpoint": "POST /api/v1/will/create",
                "parameters": ["creator_id", "heartbeat_interval_days", "grace_period_days", "viewers"],
                "auth_required": true
            },
            "capsule_create": {
                "description": "Create time capsule",
                "endpoint": "POST /api/v1/capsule/create",
                "parameters": ["creator_id", "open_at", "viewers", "timezone"],
                "auth_required": true
            },
            "chain_verify": {
                "description": "Verify on-chain timestamp proof",
                "endpoint": "GET /api/v1/chain/verify/{tape_id}",
                "parameters": ["tape_id"],
                "auth_required": false
            },
            "chain_batch_submit": {
                "description": "Submit pending hashes for on-chain batching",
                "endpoint": "POST /api/v1/chain/submit",
                "parameters": ["force"],
                "auth_required": true
            },
            "identity_card": {
                "description": "Get OpenLink Identity Card for a tape",
                "endpoint": "GET /api/v1/openlink/identity-card/{tape_id}",
                "parameters": ["tape_id"],
                "auth_required": false
            },
            "verify_identity_card": {
                "description": "Verify an Identity Card's integrity",
                "endpoint": "POST /api/v1/openlink/verify/{tape_id}",
                "parameters": ["tape_id", "content_hash"],
                "auth_required": false
            },
            "vault_create_ref": {
                "description": "Create vault file reference for large files",
                "endpoint": "POST /api/v1/vault/ref",
                "parameters": ["tape_id", "vault_file_id", "key_shares"],
                "auth_required": true
            },
            "vault_retrieve": {
                "description": "Retrieve and decrypt vault file",
                "endpoint": "POST /api/v1/vault/retrieve/{tape_id}",
                "parameters": ["tape_id", "shares"],
                "auth_required": true
            }
        },
        "endpoints": {
            "health": "GET /api/v1/health",
            "openapi": "GET /api/v1/openapi.json",
            "websocket": "WS /api/v1/ws/notifications",
            "agent_discovery": "GET /.well-known/agent.json"
        },
        "authentication": {
            "type": "bearer",
            "description": "JWT Bearer token in Authorization header",
            "obtain": "POST /api/v1/account/login",
            "refresh": "POST /api/v1/account/refresh"
        },
        "rate_limiting": {
            "description": "Rate limiting applied per capability",
            "default": "100 req/min",
            "agent": "1000 req/min"
        },
        "openmind_integration": {
            "description": "Agent can search knowledge base for seal records",
            "status": "placeholder",
            "endpoint": "POST /api/v1/openmind/search",
            "parameters": ["query", "filters"]
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
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
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

    #[tokio::test]
    async fn test_openapi_spec_endpoint() {
        let app = app(AppState::new());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
