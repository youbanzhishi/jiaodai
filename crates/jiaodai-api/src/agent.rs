//! Agent Action Protocol implementation
//!
//! Phase 12: Agent Action Protocol
//! - Complete /.well-known/agent.json definition
//! - Agent can: create seal, query status, trigger unseal, verify certificate
//! - Action middleware: authentication, rate limiting, logging
//! - OpenMind integration placeholder (agent can search knowledge base)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Complete agent.json definition following the Agent Action Protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    /// Agent name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// API version
    pub version: String,
    /// Protocol version
    pub protocol_version: String,
    /// Capabilities the agent supports
    pub capabilities: std::collections::HashMap<String, AgentCapability>,
    /// Available endpoints
    pub endpoints: std::collections::HashMap<String, String>,
    /// Authentication requirements
    pub authentication: AgentAuthentication,
    /// Rate limiting info
    pub rate_limiting: AgentRateLimiting,
    /// OpenMind integration
    pub openmind_integration: OpenMindIntegration,
}

/// A single agent capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    /// Human-readable description
    pub description: String,
    /// API endpoint
    pub endpoint: String,
    /// Required parameters
    pub parameters: Vec<String>,
    /// Whether authentication is required
    pub auth_required: bool,
    /// HTTP method
    pub method: String,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAuthentication {
    /// Auth type (bearer, api_key, etc.)
    #[serde(rename = "type")]
    pub auth_type: String,
    /// Description
    pub description: String,
    /// How to obtain auth
    pub obtain: String,
    /// How to refresh auth
    pub refresh: String,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRateLimiting {
    /// Description
    pub description: String,
    /// Default rate limit
    pub default: String,
    /// Agent rate limit
    pub agent: String,
}

/// OpenMind integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenMindIntegration {
    /// Description
    pub description: String,
    /// Status
    pub status: String,
    /// Endpoint
    pub endpoint: String,
    /// Parameters
    pub parameters: Vec<String>,
}

/// Agent action request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentActionRequest {
    /// The action to perform
    pub action: String,
    /// Action parameters
    pub parameters: serde_json::Value,
    /// Request ID for tracking
    pub request_id: Option<String>,
}

/// Agent action response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentActionResponse {
    /// Whether the action succeeded
    pub success: bool,
    /// The action performed
    pub action: String,
    /// Response data
    pub data: serde_json::Value,
    /// Request ID (echoed back)
    pub request_id: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Action log entry for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionLogEntry {
    /// Log entry ID
    pub id: String,
    /// The agent that performed the action
    pub agent_id: String,
    /// The action performed
    pub action: String,
    /// Whether it succeeded
    pub success: bool,
    /// When it happened
    pub timestamp: DateTime<Utc>,
    /// IP address
    pub ip_address: String,
    /// Request ID
    pub request_id: Option<String>,
}

/// Action middleware: authentication, rate limiting, logging
pub struct ActionMiddleware {
    /// Whether auth is required for actions
    pub auth_required: bool,
    /// Rate limit (requests per minute)
    pub rate_limit: u32,
    /// Action log
    pub log: std::sync::Mutex<Vec<ActionLogEntry>>,
}

impl ActionMiddleware {
    /// Create new action middleware
    pub fn new(auth_required: bool, rate_limit: u32) -> Self {
        Self {
            auth_required,
            rate_limit,
            log: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Default middleware for production
    pub fn production() -> Self {
        Self::new(true, 1000)
    }

    /// Default middleware for development
    pub fn development() -> Self {
        Self::new(false, 10000)
    }

    /// Log an action
    pub fn log_action(
        &self,
        agent_id: &str,
        action: &str,
        success: bool,
        ip_address: &str,
        request_id: Option<&str>,
    ) {
        let entry = ActionLogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            action: action.to_string(),
            success,
            timestamp: Utc::now(),
            ip_address: ip_address.to_string(),
            request_id: request_id.map(|s| s.to_string()),
        };
        self.log.lock().unwrap().push(entry);
    }

    /// Get recent action log entries
    pub fn get_recent_logs(&self, count: usize) -> Vec<ActionLogEntry> {
        let log = self.log.lock().unwrap();
        log.iter().rev().take(count).cloned().collect()
    }

    /// Check if rate limit is exceeded (simple in-memory check)
    pub fn check_rate_limit(&self, agent_id: &str) -> bool {
        let log = self.log.lock().unwrap();
        let one_minute_ago = Utc::now() - chrono::Duration::seconds(60);
        let recent_count = log
            .iter()
            .filter(|e| e.agent_id == agent_id && e.timestamp > one_minute_ago)
            .count();
        (recent_count as u32) < self.rate_limit
    }
}

/// Build the complete agent.json definition
pub fn build_agent_definition() -> AgentDefinition {
    let mut capabilities = std::collections::HashMap::new();

    capabilities.insert(
        "seal".to_string(),
        AgentCapability {
            description: "Create a sealed tape with encrypted content".to_string(),
            endpoint: "POST /api/v1/seal".to_string(),
            parameters: vec![
                "content_type".into(),
                "trigger_condition".into(),
                "viewers".into(),
            ],
            auth_required: true,
            method: "POST".to_string(),
        },
    );

    capabilities.insert(
        "unseal".to_string(),
        AgentCapability {
            description: "Attempt to unseal a tape".to_string(),
            endpoint: "POST /api/v1/unseal/{id}".to_string(),
            parameters: vec!["identity_claim".into()],
            auth_required: true,
            method: "POST".to_string(),
        },
    );

    capabilities.insert(
        "verify".to_string(),
        AgentCapability {
            description: "Verify a tape's integrity and chain proof".to_string(),
            endpoint: "GET /api/v1/tape/{id}/verify".to_string(),
            parameters: vec!["tape_id".into()],
            auth_required: false,
            method: "GET".to_string(),
        },
    );

    capabilities.insert(
        "status".to_string(),
        AgentCapability {
            description: "Get tape status".to_string(),
            endpoint: "GET /api/v1/tape/{id}/status".to_string(),
            parameters: vec!["tape_id".into()],
            auth_required: true,
            method: "GET".to_string(),
        },
    );

    capabilities.insert(
        "match".to_string(),
        AgentCapability {
            description: "Check for mutual match".to_string(),
            endpoint: "GET /api/v1/match/check".to_string(),
            parameters: vec!["tape_id".into()],
            auth_required: true,
            method: "GET".to_string(),
        },
    );

    capabilities.insert(
        "crush_search".to_string(),
        AgentCapability {
            description: "Search phone number for crush scenario".to_string(),
            endpoint: "POST /api/v1/crush/search".to_string(),
            parameters: vec!["phone".into()],
            auth_required: true,
            method: "POST".to_string(),
        },
    );

    capabilities.insert(
        "will_create".to_string(),
        AgentCapability {
            description: "Create will with heartbeat trigger".to_string(),
            endpoint: "POST /api/v1/will/create".to_string(),
            parameters: vec![
                "creator_id".into(),
                "heartbeat_interval_days".into(),
                "grace_period_days".into(),
                "viewers".into(),
            ],
            auth_required: true,
            method: "POST".to_string(),
        },
    );

    capabilities.insert(
        "capsule_create".to_string(),
        AgentCapability {
            description: "Create time capsule".to_string(),
            endpoint: "POST /api/v1/capsule/create".to_string(),
            parameters: vec![
                "creator_id".into(),
                "open_at".into(),
                "viewers".into(),
                "timezone".into(),
            ],
            auth_required: true,
            method: "POST".to_string(),
        },
    );

    capabilities.insert(
        "chain_verify".to_string(),
        AgentCapability {
            description: "Verify on-chain timestamp proof".to_string(),
            endpoint: "GET /api/v1/chain/verify/{tape_id}".to_string(),
            parameters: vec!["tape_id".into()],
            auth_required: false,
            method: "GET".to_string(),
        },
    );

    capabilities.insert(
        "identity_card".to_string(),
        AgentCapability {
            description: "Get OpenLink Identity Card for a tape".to_string(),
            endpoint: "GET /api/v1/openlink/identity-card/{tape_id}".to_string(),
            parameters: vec!["tape_id".into()],
            auth_required: false,
            method: "GET".to_string(),
        },
    );

    capabilities.insert(
        "verify_identity_card".to_string(),
        AgentCapability {
            description: "Verify an Identity Card's integrity".to_string(),
            endpoint: "POST /api/v1/openlink/verify/{tape_id}".to_string(),
            parameters: vec!["tape_id".into(), "content_hash".into()],
            auth_required: false,
            method: "POST".to_string(),
        },
    );

    capabilities.insert(
        "vault_create_ref".to_string(),
        AgentCapability {
            description: "Create vault file reference for large files".to_string(),
            endpoint: "POST /api/v1/vault/ref".to_string(),
            parameters: vec![
                "tape_id".into(),
                "vault_file_id".into(),
                "key_shares".into(),
            ],
            auth_required: true,
            method: "POST".to_string(),
        },
    );

    capabilities.insert(
        "vault_retrieve".to_string(),
        AgentCapability {
            description: "Retrieve and decrypt vault file".to_string(),
            endpoint: "POST /api/v1/vault/retrieve/{tape_id}".to_string(),
            parameters: vec!["tape_id".into(), "shares".into()],
            auth_required: true,
            method: "POST".to_string(),
        },
    );

    let mut endpoints = std::collections::HashMap::new();
    endpoints.insert("health".to_string(), "GET /api/v1/health".to_string());
    endpoints.insert(
        "openapi".to_string(),
        "GET /api/v1/openapi.json".to_string(),
    );
    endpoints.insert(
        "websocket".to_string(),
        "WS /api/v1/ws/notifications".to_string(),
    );
    endpoints.insert(
        "agent_discovery".to_string(),
        "GET /.well-known/agent.json".to_string(),
    );

    AgentDefinition {
        name: "jiaodai".to_string(),
        description: "时间封存平台 - Seal now, open when conditions met".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        protocol_version: "1.0.0".to_string(),
        capabilities,
        endpoints,
        authentication: AgentAuthentication {
            auth_type: "bearer".to_string(),
            description: "JWT Bearer token in Authorization header".to_string(),
            obtain: "POST /api/v1/account/login".to_string(),
            refresh: "POST /api/v1/account/refresh".to_string(),
        },
        rate_limiting: AgentRateLimiting {
            description: "Rate limiting applied per capability".to_string(),
            default: "100 req/min".to_string(),
            agent: "1000 req/min".to_string(),
        },
        openmind_integration: OpenMindIntegration {
            description: "Agent can search knowledge base for seal records".to_string(),
            status: "placeholder".to_string(),
            endpoint: "POST /api/v1/openmind/search".to_string(),
            parameters: vec!["query".into(), "filters".into()],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_agent_definition() {
        let def = build_agent_definition();
        assert_eq!(def.name, "jiaodai");
        assert!(!def.capabilities.is_empty());
        assert!(def.capabilities.contains_key("seal"));
        assert!(def.capabilities.contains_key("unseal"));
        assert!(def.capabilities.contains_key("verify"));
        assert!(def.capabilities.contains_key("identity_card"));
        assert!(def.capabilities.contains_key("vault_create_ref"));
    }

    #[test]
    fn test_agent_definition_serialization() {
        let def = build_agent_definition();
        let json = serde_json::to_string(&def).unwrap();
        let parsed: AgentDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "jiaodai");
        assert_eq!(parsed.capabilities.len(), def.capabilities.len());
    }

    #[test]
    fn test_action_middleware_logging() {
        let mw = ActionMiddleware::development();
        mw.log_action("agent-1", "seal", true, "127.0.0.1", Some("req-1"));
        mw.log_action("agent-1", "verify", true, "127.0.0.1", None);
        mw.log_action("agent-2", "seal", false, "10.0.0.1", Some("req-2"));

        let logs = mw.get_recent_logs(10);
        assert_eq!(logs.len(), 3);

        let agent1_logs = mw
            .get_recent_logs(10)
            .into_iter()
            .filter(|l| l.agent_id == "agent-1")
            .count();
        assert_eq!(agent1_logs, 2);
    }

    #[test]
    fn test_action_middleware_rate_limit() {
        let mw = ActionMiddleware::new(true, 5);
        // Under limit
        assert!(mw.check_rate_limit("agent-1"));

        // Exceed limit
        for i in 0..5 {
            mw.log_action("agent-1", "seal", true, "127.0.0.1", None);
        }
        assert!(!mw.check_rate_limit("agent-1"));

        // Different agent should be fine
        assert!(mw.check_rate_limit("agent-2"));
    }

    #[test]
    fn test_agent_action_request() {
        let req = AgentActionRequest {
            action: "seal".to_string(),
            parameters: serde_json::json!({"content_type": "text"}),
            request_id: Some("req-1".to_string()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("seal"));
    }

    #[test]
    fn test_agent_action_response() {
        let resp = AgentActionResponse {
            success: true,
            action: "seal".to_string(),
            data: serde_json::json!({"tape_id": "tape-1"}),
            request_id: Some("req-1".to_string()),
            timestamp: Utc::now(),
        };
        assert!(resp.success);
    }

    #[test]
    fn test_capability_auth_requirements() {
        let def = build_agent_definition();
        // Public endpoints should not require auth
        assert!(!def.capabilities["verify"].auth_required);
        assert!(!def.capabilities["chain_verify"].auth_required);
        assert!(!def.capabilities["identity_card"].auth_required);
        assert!(!def.capabilities["verify_identity_card"].auth_required);

        // Protected endpoints should require auth
        assert!(def.capabilities["seal"].auth_required);
        assert!(def.capabilities["unseal"].auth_required);
        assert!(def.capabilities["status"].auth_required);
        assert!(def.capabilities["vault_create_ref"].auth_required);
        assert!(def.capabilities["vault_retrieve"].auth_required);
    }

    #[test]
    fn test_openmind_integration() {
        let def = build_agent_definition();
        assert_eq!(def.openmind_integration.status, "placeholder");
        assert_eq!(
            def.openmind_integration.endpoint,
            "POST /api/v1/openmind/search"
        );
    }
}
