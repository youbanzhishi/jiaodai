//! # jiaodai-match
//!
//! The mutual-match engine for the Jiaodai platform (暗恋表白 scenario).
//!
//! Hash matching: hash(A→B's phone) vs B's account_id
//! Only when both directions match, both parties are notified simultaneously.
//! If only one direction exists, there is zero information leakage.

mod engine;
mod passive;
mod search;

pub use engine::*;
pub use passive::*;
// Re-export specific items from search to avoid ambiguous glob
pub use search::{phone_hash, PhoneSearchService, SearchResult};
