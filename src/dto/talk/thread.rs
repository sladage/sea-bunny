use serde::Deserialize;

use super::{ChatMessage, NotificationLevel};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Thread {
    /// Id of the message that started the thread.
    pub id: i64,
    pub room_token: String,
    pub title: String,
    pub last_message_id: i64,
    pub last_activity: i64,
    pub num_replies: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ThreadAttendee {
    pub notification_level: NotificationLevel,
}

/// A thread with the current user's settings and its first and last message.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ThreadInfo {
    pub thread: Thread,
    pub attendee: ThreadAttendee,
    pub first: Option<ChatMessage>,
    pub last: Option<ChatMessage>,
}
