//! Time Capsule (时间胶囊) scene
//!
//! Flow:
//! 1. Creator creates a capsule with:
//!    - Content: encrypted message
//!    - Trigger: DateTrigger (specific open date)
//!    - Viewers: specified people (or self)
//! 2. Short link/QR code generated for sharing
//! 3. Viewers can see countdown (content not visible until open date)
//! 4. On open date → auto-unseal → notify viewers
//! 5. Self-capsule: viewer = creator themselves

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use jiaodai_core::Result;
use jiaodai_seal::certificate::{CertificateManager, ShareMethod};

/// Time capsule creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleRequest {
    /// Creator's account ID
    pub creator_id: String,
    /// Open date (UTC)
    pub open_at: DateTime<Utc>,
    /// Viewers (can include self)
    pub viewers: Vec<CapsuleViewer>,
    /// Timezone for display
    pub timezone: String,
}

/// A viewer for a time capsule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleViewer {
    /// Viewer identifier (account ID or phone hash)
    pub identifier: String,
    /// Viewer type
    pub viewer_type: CapsuleViewerType,
    /// Display name (for countdown page)
    pub display_name: Option<String>,
}

/// Type of capsule viewer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleViewerType {
    Account,
    PhoneHash,
    Anyone,
    Self_,
}

/// A time capsule record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleRecord {
    /// The tape ID
    pub tape_id: String,
    /// Creator's account ID
    pub creator_id: String,
    /// Open date
    pub open_at: DateTime<Utc>,
    /// Viewers
    pub viewers: Vec<CapsuleViewer>,
    /// Timezone
    pub timezone: String,
    /// Status
    pub status: CapsuleStatus,
    /// Share info
    pub share: Option<CapsuleShareInfo>,
    /// Created at
    pub created_at: DateTime<Utc>,
}

/// Status of a time capsule
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleStatus {
    /// Sealed, waiting for open date
    Sealed,
    /// Open date reached, content available
    Opened,
    /// Archived
    Archived,
}

/// Share information for a capsule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleShareInfo {
    /// Short link
    pub short_link: String,
    /// QR code data
    pub qr_data: String,
    /// Share method
    pub method: String,
}

/// Time capsule scene service
pub struct CapsuleScene {
    capsules: std::sync::Mutex<Vec<CapsuleRecord>>,
}

impl CapsuleScene {
    /// Create a new capsule scene service
    pub fn new() -> Self {
        Self {
            capsules: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Create a time capsule
    pub fn create_capsule(&self, request: CapsuleRequest) -> Result<CapsuleRecord> {
        let tape_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Generate share info
        let share = CertificateManager::generate_share(&tape_id, ShareMethod::ShortLink);
        let share_info = CapsuleShareInfo {
            short_link: share.short_link,
            qr_data: share.qr_data,
            method: "short_link".to_string(),
        };

        let record = CapsuleRecord {
            tape_id,
            creator_id: request.creator_id,
            open_at: request.open_at,
            viewers: request.viewers,
            timezone: request.timezone,
            status: CapsuleStatus::Sealed,
            share: Some(share_info),
            created_at: now,
        };

        self.capsules.lock().unwrap().push(record.clone());
        Ok(record)
    }

    /// Create a self-capsule (for yourself in the future)
    pub fn create_self_capsule(
        &self,
        creator_id: &str,
        open_at: DateTime<Utc>,
        timezone: &str,
    ) -> Result<CapsuleRecord> {
        self.create_capsule(CapsuleRequest {
            creator_id: creator_id.to_string(),
            open_at,
            viewers: vec![CapsuleViewer {
                identifier: creator_id.to_string(),
                viewer_type: CapsuleViewerType::Self_,
                display_name: Some("Myself".to_string()),
            }],
            timezone: timezone.to_string(),
        })
    }

    /// Get countdown for a capsule (days remaining)
    pub fn get_countdown(&self, tape_id: &str) -> Option<i64> {
        let capsules = self.capsules.lock().unwrap();
        let capsule = capsules.iter().find(|c| c.tape_id == tape_id)?;
        let remaining = (capsule.open_at - Utc::now()).num_days();
        Some(remaining)
    }

    /// Check and open capsules that have reached their open date
    pub fn check_and_open(&self) -> Vec<CapsuleRecord> {
        let now = Utc::now();
        let mut capsules = self.capsules.lock().unwrap();
        let mut opened = Vec::new();

        for capsule in capsules.iter_mut() {
            if capsule.status == CapsuleStatus::Sealed && now >= capsule.open_at {
                capsule.status = CapsuleStatus::Opened;
                opened.push(capsule.clone());
            }
        }

        opened
    }

    /// Get a capsule record
    pub fn get_capsule(&self, tape_id: &str) -> Option<CapsuleRecord> {
        let capsules = self.capsules.lock().unwrap();
        capsules.iter().find(|c| c.tape_id == tape_id).cloned()
    }

    /// Get all capsules for a creator
    pub fn get_creator_capsules(&self, creator_id: &str) -> Vec<CapsuleRecord> {
        let capsules = self.capsules.lock().unwrap();
        capsules
            .iter()
            .filter(|c| c.creator_id == creator_id)
            .cloned()
            .collect()
    }
}

impl Default for CapsuleScene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_capsule() {
        let scene = CapsuleScene::new();
        let request = CapsuleRequest {
            creator_id: "account-1".to_string(),
            open_at: Utc::now() + chrono::Duration::days(365),
            viewers: vec![CapsuleViewer {
                identifier: "account-2".to_string(),
                viewer_type: CapsuleViewerType::Account,
                display_name: Some("Bob".to_string()),
            }],
            timezone: "Asia/Shanghai".to_string(),
        };
        let capsule = scene.create_capsule(request).unwrap();
        assert_eq!(capsule.status, CapsuleStatus::Sealed);
        assert!(capsule.share.is_some());
    }

    #[test]
    fn test_create_self_capsule() {
        let scene = CapsuleScene::new();
        let capsule = scene
            .create_self_capsule(
                "account-1",
                Utc::now() + chrono::Duration::days(365),
                "Asia/Shanghai",
            )
            .unwrap();
        assert_eq!(capsule.viewers.len(), 1);
        assert_eq!(capsule.viewers[0].viewer_type, CapsuleViewerType::Self_);
    }

    #[test]
    fn test_countdown() {
        let scene = CapsuleScene::new();
        let capsule = scene
            .create_self_capsule(
                "account-1",
                Utc::now() + chrono::Duration::days(365),
                "Asia/Shanghai",
            )
            .unwrap();
        let countdown = scene.get_countdown(&capsule.tape_id);
        assert!(countdown.is_some());
        assert!(countdown.unwrap() > 360);
    }

    #[test]
    fn test_check_and_open_future() {
        let scene = CapsuleScene::new();
        scene
            .create_self_capsule(
                "account-1",
                Utc::now() + chrono::Duration::days(365),
                "Asia/Shanghai",
            )
            .unwrap();
        let opened = scene.check_and_open();
        assert!(opened.is_empty());
    }

    #[test]
    fn test_check_and_open_past() {
        let scene = CapsuleScene::new();
        scene
            .create_self_capsule(
                "account-1",
                Utc::now() - chrono::Duration::days(1),
                "Asia/Shanghai",
            )
            .unwrap();
        let opened = scene.check_and_open();
        assert_eq!(opened.len(), 1);
        assert_eq!(opened[0].status, CapsuleStatus::Opened);
    }
}
