//! Data models for the Jiaodai platform
//!
//! Reference: Blueprint Chapter 9 (Data Models)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Content Types ───────────────────────────────────────────────

/// The type of content sealed in a tape
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Text,
    Image,
    Video,
    File,
    VaultRef,
}

// ─── Tape Status ─────────────────────────────────────────────────

/// The lifecycle status of a tape
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TapeStatus {
    Draft,
    Sealed,
    Partial,
    Triggered,
    Grace,
    Unsealed,
    Archived,
}

impl TapeStatus {
    /// Check if a transition to the target status is valid
    pub fn can_transition_to(&self, target: &TapeStatus) -> bool {
        matches!(
            (self, target),
            (TapeStatus::Draft, TapeStatus::Sealed)
            | (TapeStatus::Sealed, TapeStatus::Partial)
            | (TapeStatus::Sealed, TapeStatus::Triggered)
            | (TapeStatus::Partial, TapeStatus::Sealed)
            | (TapeStatus::Triggered, TapeStatus::Grace)
            | (TapeStatus::Triggered, TapeStatus::Unsealed)
            | (TapeStatus::Grace, TapeStatus::Unsealed)
            | (TapeStatus::Grace, TapeStatus::Sealed)
            | (TapeStatus::Unsealed, TapeStatus::Archived)
            | (TapeStatus::Sealed, TapeStatus::Archived)
        )
    }
}

// ─── Tape (封存物) ──────────────────────────────────────────────

/// A sealed tape — the core entity of the platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tape {
    pub id: String,
    pub creator_id: String,
    pub content_type: ContentType,
    pub encrypted_content: Vec<u8>,
    pub content_hash: [u8; 32],
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub status: TapeStatus,
    pub created_at: DateTime<Utc>,
    pub sealed_at: Option<DateTime<Utc>>,
    pub unsealed_at: Option<DateTime<Utc>>,
}

// ─── Account (账号) ─────────────────────────────────────────────

/// Account status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    Active,
    Suspended,
    Deleted,
}

/// A phone number binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneBind {
    pub id: String,
    pub account_id: String,
    pub phone_hash: String,
    pub phone_encrypted: String,
    pub is_primary: bool,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

/// Identity information (encrypted, for recovery)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityInfo {
    pub id: String,
    pub account_id: String,
    pub id_number_hash: String,
    pub name_encrypted: String,
    pub id_number_encrypted: String,
    pub verified: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// A user account — the permanent anchor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub phone_numbers: Vec<PhoneBind>,
    pub identity: Option<IdentityInfo>,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Trigger Condition (解封条件) ──────────────────────────────

/// Logic operator for composite conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogicOp {
    And,
    Or,
}

/// A confirmer — someone who can confirm the creator is still alive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Confirmer {
    pub account_id: Option<String>,
    pub phone_hash: Option<String>,
    pub name: String,
    pub last_confirmed_at: Option<DateTime<Utc>>,
}

/// The type of trigger condition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {

    Heartbeat,
    MutualMatch,
    DateTrigger,
    MultiConfirm,
    Composite,
}

/// The various trigger conditions for unsealing a tape
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerCondition {
    /// Heartbeat timeout: if the creator's confirmers don't confirm within N days
    Heartbeat {
        timeout_days: u32,
        confirmers: Vec<Confirmer>,
    },
    /// Mutual match: both A→B and B→A have sealed tapes
    MutualMatch {
        target_account_id: String,
    },
    /// Date trigger: open at a specific date/time
    DateTrigger {
        open_at: DateTime<Utc>,
    },
    /// Multi-person confirmation: M of N confirmers must agree
    MultiConfirm {
        threshold: u32,
        confirmers: Vec<Confirmer>,
    },
    /// Composite: multiple conditions combined with AND/OR logic
    Composite {
        conditions: Vec<TriggerCondition>,
        logic: LogicOp,
    },
}

impl TriggerCondition {
    /// Get the condition type
    pub fn condition_type(&self) -> ConditionType {
        match self {
            TriggerCondition::Heartbeat { .. } => ConditionType::Heartbeat,
            TriggerCondition::MutualMatch { .. } => ConditionType::MutualMatch,
            TriggerCondition::DateTrigger { .. } => ConditionType::DateTrigger,
            TriggerCondition::MultiConfirm { .. } => ConditionType::MultiConfirm,
            TriggerCondition::Composite { .. } => ConditionType::Composite,
        }
    }
}

// ─── Viewer (查看人) ────────────────────────────────────────────

/// The type of viewer identity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViewerType {
    Account,
    PhoneHash,
    Identity,
    Anyone,
}

/// Who can view the unsealed content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Viewer {
    /// A specific account holder
    Account { account_id: String },
    /// Someone identified by phone number hash
    PhoneHash { phone_hash: String },
    /// Someone identified by identity number hash
    Identity { id_number_hash: String },
    /// Anyone can view
    Anyone,
}

impl Viewer {
    /// Get the viewer type
    pub fn viewer_type(&self) -> ViewerType {
        match self {
            Viewer::Account { .. } => ViewerType::Account,
            Viewer::PhoneHash { .. } => ViewerType::PhoneHash,
            Viewer::Identity { .. } => ViewerType::Identity,
            Viewer::Anyone => ViewerType::Anyone,
        }
    }
}

// ─── Seal Certificate (封存凭证) ────────────────────────────────

/// A seal certificate proving the tape was sealed at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealCertificate {
    pub tape_id: String,
    pub sealed_at: DateTime<Utc>,
    pub content_hash: [u8; 32],
    pub chain_tx_hash: Option<String>,
    pub chain_block_number: Option<u64>,
    pub trigger_condition: TriggerCondition,
    pub viewers: Vec<Viewer>,
}

// ─── Trigger Event (触发事件) ───────────────────────────────────

/// The type of trigger event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerEventType {
    HeartbeatTimeout,
    MatchFound,
    DateReached,
    ConfirmReceived,
}

/// A trigger event recording state changes in the unseal process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEvent {
    pub id: String,
    pub tape_id: String,
    pub event_type: TriggerEventType,
    pub event_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ─── Blockchain Attestation (区块链证明) ────────────────────────

/// A blockchain attestation for a sealed tape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainAttestation {
    pub id: String,
    pub tape_id: String,
    pub content_hash: String,
    pub merkle_root: String,
    pub merkle_proof: Vec<String>,
    pub root_index: u64,
    pub tx_hash: String,
    pub block_number: u64,
    pub timestamp: i64,
    pub network: String,
    pub created_at: DateTime<Utc>,
}

// ─── Condition State (条件状态) ─────────────────────────────────

/// The state of a condition check
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionState {
    /// Condition is not yet met
    Pending,
    /// Condition is partially met (e.g., some confirmers confirmed)
    Partial,
    /// Condition is fully met
    Satisfied,
    /// Condition check failed
    Failed(String),
}

// ─── Trigger Context ────────────────────────────────────────────

/// Context provided when checking trigger conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerContext {
    pub tape_id: String,
    pub current_time: DateTime<Utc>,
    pub heartbeat_last_at: Option<DateTime<Utc>>,
    pub confirmed_count: Option<u32>,
    pub total_confirmers: Option<u32>,
}

// ─── Identity Claim ─────────────────────────────────────────────

/// An identity claim for viewer verification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IdentityClaim {
    Account { account_id: String },
    Phone { phone_hash: String },
    Identity { id_number_hash: String },
    Anonymous,
}

// ─── Key Share ──────────────────────────────────────────────────

/// A Shamir's Secret Sharing key share
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyShare {
    pub index: u8,
    pub data: Vec<u8>,
}

// ─── Encrypted Content ─────────────────────────────────────────

/// Encrypted content with metadata for AES-256-GCM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedContent {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub tag: [u8; 16],
}

// ─── Heartbeat ──────────────────────────────────────────────────

/// A heartbeat record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub id: String,
    pub account_id: String,
    pub beat_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
