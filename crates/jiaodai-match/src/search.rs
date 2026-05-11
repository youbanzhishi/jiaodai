//! Phone number search functionality
//!
//! Search for users by phone number hash. This is the entry point
//! for the crush (暗恋表白) scenario.
//!
//! Privacy: searches leave no trace. The searched party never knows
//! they were searched for.

use sha2::{Digest, Sha256};

use jiaodai_core::JiaodaiError;

/// Result of a phone number search
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Whether the phone number is registered
    pub registered: bool,
    /// The phone number hash (for matching)
    pub phone_hash: String,
    /// Account ID if registered (None for privacy in some contexts)
    pub account_id: Option<String>,
}

/// Phone number search service
pub struct PhoneSearchService {
    /// Registered phone hashes → account IDs
    registrations: std::sync::Mutex<Vec<(String, String)>>,
}

impl PhoneSearchService {
    /// Create a new search service
    pub fn new() -> Self {
        Self {
            registrations: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Search for a phone number
    ///
    /// Returns whether the phone is registered and its hash.
    /// The search is traceless — no log is kept.
    pub fn search(&self, phone: &str) -> SearchResult {
        let hash = phone_hash(phone);
        let regs = self.registrations.lock().unwrap();
        let found = regs.iter().find(|(h, _)| h == &hash);
        SearchResult {
            registered: found.is_some(),
            phone_hash: hash,
            account_id: found.map(|(_, id)| id.clone()),
        }
    }

    /// Register a phone number (called during account creation)
    pub fn register(&self, phone: &str, account_id: &str) {
        let hash = phone_hash(phone);
        let mut regs = self.registrations.lock().unwrap();
        // Don't duplicate
        if !regs.iter().any(|(h, _)| h == &hash) {
            regs.push((hash, account_id.to_string()));
        }
    }

    /// Check if a phone is registered
    pub fn is_registered(&self, phone: &str) -> bool {
        let hash = phone_hash(phone);
        let regs = self.registrations.lock().unwrap();
        regs.iter().any(|(h, _)| h == &hash)
    }
}

impl Default for PhoneSearchService {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a SHA-256 hash of a phone number for search/matching
pub fn phone_hash(phone: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(phone.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_not_registered() {
        let service = PhoneSearchService::new();
        let result = service.search("13800138000");
        assert!(!result.registered);
    }

    #[test]
    fn test_search_registered() {
        let service = PhoneSearchService::new();
        service.register("13800138000", "account-1");
        let result = service.search("13800138000");
        assert!(result.registered);
        assert_eq!(result.account_id, Some("account-1".to_string()));
    }

    #[test]
    fn test_search_traceless() {
        // Search doesn't store any trace — just returns result
        let service = PhoneSearchService::new();
        let result = service.search("13900139000");
        assert!(!result.registered);
        // No way to know who searched or when
    }

    #[test]
    fn test_register_duplicate() {
        let service = PhoneSearchService::new();
        service.register("13800138000", "account-1");
        service.register("13800138000", "account-1"); // No duplicate
    }
}
