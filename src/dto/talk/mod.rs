//! Data transfer objects for the Nextcloud Talk (spreed) API.
//!
//! Response types use `#[serde(default)]` at struct level: fields Talk only sends
//! with certain capabilities, or that older servers lack, fall back to defaults
//! instead of failing the whole response.

pub mod chat;
pub mod constants;
pub mod conversation;
pub mod participant;
pub mod user_status;

pub use chat::*;
pub use constants::*;
pub use conversation::*;
pub use participant::*;
pub use user_status::*;
