//! Data transfer objects for the Nextcloud Talk (spreed) API.
//!
//! Response types use `#[serde(default)]` at struct level: fields Talk only sends
//! with certain capabilities, or that older servers lack, fall back to defaults
//! instead of failing the whole response.

pub mod ban;
pub mod bot;
pub mod breakout;
pub mod calendar;
pub mod call;
pub mod chat;
pub mod constants;
pub mod conversation;
pub mod federation;
pub mod matterbridge;
pub mod participant;
pub mod poll;
pub mod preset;
pub mod reaction;
pub mod reminder;
pub mod scheduled;
pub mod sharing;
pub mod signaling;
pub mod tag;
pub mod thread;
pub mod transcription;
pub mod user_settings;
pub mod user_status;

pub use ban::*;
pub use bot::*;
pub use breakout::*;
pub use calendar::*;
pub use call::*;
pub use chat::*;
pub use constants::*;
pub use conversation::*;
pub use federation::*;
pub use matterbridge::*;
pub use participant::*;
pub use poll::*;
pub use preset::*;
pub use reaction::*;
pub use reminder::*;
pub use scheduled::*;
pub use sharing::*;
pub use signaling::*;
pub use tag::*;
pub use thread::*;
pub use transcription::*;
pub use user_settings::*;
pub use user_status::*;

pub use crate::dto::rich_object::RichObjectParameter;
