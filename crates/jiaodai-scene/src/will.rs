//! Will (遗嘱交代) scene
//!
//! Flow:
//! 1. Creator completes identity verification (real-name + ID card)
//! 2. Creator creates a will tape with:
//!    - Content: encrypted will
//!    - Trigger: Heartbeat timeout (configurable: 7/14/30/90 days)
//!    - Viewers: specified heirs (identified by identity number hash)
//! 3. Creator sends regular heartbeats
//! 4. If heartbeat stops → grace period → unseal → notify viewers
//! 5. Viewers must verify identity (real-name + ID match) to view content

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use jiaodai_core::Result;

/// Will creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillRequest {
    /// Creator's account ID (must be identity-verified)
    pub creator_id: String,
    /// Heartbeat interval in days (7/14/30/90)
    pub heartbeat_interval_days: u32,
    /// Grace period in days after heartbeat timeout
    pub grace_period_days: u32,
    /// Viewers (heirs) identified by identity number hash
    pub viewers: Vec<WillViewer>,
    /// Will content (will be encrypted client-side)
    pub content_preview: Option<String>,
}

/// A viewer for a will
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillViewer {
    /// Viewer's name (encrypted)
    pub name: String,
    /// Viewer's identity number hash (for verification)
    pub id_number_hash: String,
    /// Relationship to creator
    pub relationship: String,
}

/// Will scene service
pub struct WillScene {
    /// In-memory heartbeat store
    heartbeats: std::sync::Mutex<Vec<(String, DateTime<Utc>)>>,
    /// In-memory will store
    wills: std::sync::Mutex<Vec<WillRecord>>,
}

/// A will record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillRecord {
    /// The tape ID
    pub tape_id: String,
    /// Creator's account ID
    pub creator_id: String,
    /// Heartbeat interval in days
    pub heartbeat_interval_days: u32,
    /// Grace period in days
    pub grace_period_days: u32,
    /// Viewers (heirs)
    pub viewers: Vec<WillViewer>,
    /// Current status
    pub status: WillStatus,
    /// When the will was created
    pub created_at: DateTime<Utc>,
    /// Last heartbeat
    pub last_heartbeat_at: Option<DateTime<Utc>>,
}

/// Status of a will
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WillStatus {
    /// Active, receiving heartbeats
    Active,
    /// Heartbeat missed, in grace period
    GracePeriod,
    /// Unsealed, viewers can access
    Unsealed,
}

impl WillScene {
    /// Create a new will scene service
    pub fn new() -> Self {
        Self {
            heartbeats: std::sync::Mutex::new(Vec::new()),
            wills: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Create a will
    pub fn create_will(&self, request: WillRequest) -> Result<WillRecord> {
        let record = WillRecord {
            tape_id: Uuid::new_v4().to_string(),
            creator_id: request.creator_id,
            heartbeat_interval_days: request.heartbeat_interval_days,
            grace_period_days: request.grace_period_days,
            viewers: request.viewers,
            status: WillStatus::Active,
            created_at: Utc::now(),
            last_heartbeat_at: Some(Utc::now()),
        };

        self.wills.lock().unwrap().push(record.clone());
        Ok(record)
    }

    /// Send a heartbeat
    pub fn send_heartbeat(&self, account_id: &str) -> Result<()> {
        let mut heartbeats = self.heartbeats.lock().unwrap();
        let now = Utc::now();
        if let Some(entry) = heartbeats.iter_mut().find(|(id, _)| id == account_id) {
            entry.1 = now;
        } else {
            heartbeats.push((account_id.to_string(), now));
        }

        // Update will records
        let mut wills = self.wills.lock().unwrap();
        for will in wills.iter_mut() {
            if will.creator_id == account_id && will.status == WillStatus::Active {
                will.last_heartbeat_at = Some(now);
            }
        }

        Ok(())
    }

    /// Check heartbeat status for all wills
    pub fn check_heartbeats(&self) -> Vec<WillRecord> {
        let now = Utc::now();
        let mut wills = self.wills.lock().unwrap();
        let mut expired = Vec::new();

        for will in wills.iter_mut() {
            if will.status != WillStatus::Active {
                continue;
            }

            if let Some(last_hb) = will.last_heartbeat_at {
                let elapsed_days = (now - last_hb).num_days() as u32;
                if elapsed_days > will.heartbeat_interval_days + will.grace_period_days {
                    will.status = WillStatus::Unsealed;
                    expired.push(will.clone());
                } else if elapsed_days > will.heartbeat_interval_days {
                    will.status = WillStatus::GracePeriod;
                }
            }
        }

        expired
    }

    /// Get a will record
    pub fn get_will(&self, tape_id: &str) -> Option<WillRecord> {
        let wills = self.wills.lock().unwrap();
        wills.iter().find(|w| w.tape_id == tape_id).cloned()
    }

    /// Get all wills for a creator
    pub fn get_creator_wills(&self, creator_id: &str) -> Vec<WillRecord> {
        let wills = self.wills.lock().unwrap();
        wills.iter().filter(|w| w.creator_id == creator_id).cloned().collect()
    }
}

impl Default for WillScene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_will() {
        let scene = WillScene::new();
        let request = WillRequest {
            creator_id: "account-1".to_string(),
            heartbeat_interval_days: 30,
            grace_period_days: 7,
            viewers: vec![WillViewer {
                name: "继承人".to_string(),
                id_number_hash: "hash-of-id-number".to_string(),
                relationship: "child".to_string(),
            }],
            content_preview: None,
        };
        let will = scene.create_will(request).unwrap();
        assert_eq!(will.status, WillStatus::Active);
        assert_eq!(will.creator_id, "account-1");
    }

    #[test]
    fn test_heartbeat() {
        let scene = WillScene::new();
        let request = WillRequest {
            creator_id: "account-1".to_string(),
            heartbeat_interval_days: 30,
            grace_period_days: 7,
            viewers: vec![],
            content_preview: None,
        };
        scene.create_will(request).unwrap();
        scene.send_heartbeat("account-1").unwrap();

        let will = scene.get_creator_wills("account-1")[0].clone();
        assert!(will.last_heartbeat_at.is_some());
    }

    #[test]
    fn test_get_creator_wills() {
        let scene = WillScene::new();
        scene.create_will(WillRequest {
            creator_id: "account-1".to_string(),
            heartbeat_interval_days: 30,
            grace_period_days: 7,
            viewers: vec![],
            content_preview: None,
        }).unwrap();
        scene.create_will(WillRequest {
            creator_id: "account-1".to_string(),
            heartbeat_interval_days: 90,
            grace_period_days: 14,
            viewers: vec![],
            content_preview: None,
        }).unwrap();

        let wills = scene.get_creator_wills("account-1");
        assert_eq!(wills.len(), 2);
    }
}
