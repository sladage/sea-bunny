use serde::{Deserialize, Serialize};

use super::{ActorType, ChatMessage, MessageType};

/// A message scheduled to be sent later. Only visible to its author.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ScheduledMessage {
    pub id: String,
    pub actor_type: ActorType,
    pub actor_id: String,
    pub message: String,
    pub message_type: MessageType,
    pub created_at: i64,
    pub send_at: i64,
    /// Set when sending was postponed (e.g. the conversation was read-only).
    pub original_send_at: Option<i64>,
    pub silent: bool,
    pub thread_id: i64,
    pub thread_title: Option<String>,
    /// The message this one will reply to.
    pub parent: Option<Box<ChatMessage>>,
}

/// Body for scheduling a message.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleMessage {
    pub message: String,
    /// Unix timestamp to send at.
    pub send_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<i64>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub silent: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_title: Option<String>,
}

/// Body for changing a scheduled message.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditScheduledMessage {
    pub message: String,
    pub send_at: i64,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub silent: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_title: Option<String>,
}
