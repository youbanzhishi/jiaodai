//! # jiaodai-scene
//!
//! Scene implementations for the Jiaodai platform.
//!
//! Scenes are combinations of trigger conditions + viewer verification,
//! not hardcoded modules (Architecture Rule #6):
//! - Crush (暗恋表白) = MutualMatch trigger + PhoneHash viewer
//! - Will (遗嘱交代) = Heartbeat trigger + Identity viewer
//! - Capsule (时间胶囊) = DateTrigger + specified viewers
//!
//! Each scene provides a convenience API that composes the
//! underlying building blocks from jiaodai-seal, jiaodai-unseal,
//! jiaodai-match, and jiaodai-auth.

pub mod capsule;
pub mod crush;
pub mod will;

pub use capsule::*;
pub use crush::*;
pub use will::*;
