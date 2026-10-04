use std::collections::HashMap;

use serde::Deserialize;

use super::{ActorType, RichObjectParameter};
use crate::dto::serde_ext::empty_as_default;

/// A reminder the current user set on a message.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ChatReminder {
    pub token: String,
    pub message_id: i64,
    /// Unix timestamp at which the reminder notification is sent.
    pub timestamp: i64,
    pub user_id: String,
}

/// An upcoming reminder across all conversations, with the message it is for.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UpcomingReminder {
    pub room_token: String,
    pub message_id: i64,
    pub reminder_timestamp: i64,
    pub actor_type: ActorType,
    pub actor_id: String,
    pub actor_display_name: String,
    pub message: String,
    #[serde(deserialize_with = "empty_as_default")]
    pub message_parameters: HashMap<String, RichObjectParameter>,
}
