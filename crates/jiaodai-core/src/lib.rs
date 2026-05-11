//! # jiaodai-core
//!
//! Core traits and data models for the Jiaodai (胶带) time-seal platform.
//!
//! The platform revolves around three key elements:
//! - **Sealable**: Content that can be sealed (封存物)
//! - **TriggerCondition**: Rules for when content can be unsealed (解封条件)
//! - **Viewer**: Who can view the content after unsealing (查看人)

mod error;
mod models;
mod traits;

pub use error::{JiaodaiError, Result};
pub use models::*;
pub use traits::*;
