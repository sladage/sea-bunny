use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{ActorType, MessageType, UserStatus};
use crate::dto::serde_ext::{bool_as_int, empty_array_as_default};

/// A chat message. Also used for deleted parents (`deleted == true`, only `id` set)
/// and for the proxy messages of federated conversations (no `id`).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: i64,
    pub token: String,
    pub actor_type: ActorType,
    pub actor_id: String,
    pub actor_display_name: String,
    pub timestamp: i64,
    /// Message text with `{placeholder}`s for each entry in `message_parameters`.
    pub message: String,
    #[serde(deserialize_with = "empty_array_as_default")]
    pub message_parameters: HashMap<String, RichObjectParameter>,
    pub message_type: MessageType,
    /// System message identifier (e.g. `conversation_created`), empty for comments.
    pub system_message: String,
    pub is_replyable: bool,
    pub markdown: bool,
    pub silent: bool,
    pub deleted: bool,
    pub reference_id: String,
    /// Unix timestamp at which the message expires, 0 when it doesn't.
    pub expiration_timestamp: i64,

    #[serde(deserialize_with = "empty_array_as_default")]
    pub reactions: HashMap<String, i64>,
    pub reactions_self: Vec<String>,

    pub last_edit_actor_type: Option<ActorType>,
    pub last_edit_actor_id: Option<String>,
    pub last_edit_actor_display_name: Option<String>,
    pub last_edit_timestamp: Option<i64>,

    pub thread_id: i64,
    pub is_thread: bool,
    pub thread_title: Option<String>,
    pub thread_replies: Option<i64>,

    pub meta_data: Option<ChatMessageMetaData>,
    /// The message this one replies to.
    pub parent: Option<Box<ChatMessage>>,
}

impl ChatMessage {
    pub fn is_system(&self) -> bool {
        self.message_type == MessageType::System
    }

    pub fn is_edited(&self) -> bool {
        self.last_edit_timestamp.is_some_and(|t| t > 0)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ChatMessageMetaData {
    pub pinned_actor_type: Option<ActorType>,
    pub pinned_actor_id: Option<String>,
    pub pinned_actor_display_name: Option<String>,
    pub pinned_at: Option<i64>,
    pub pinned_until: Option<i64>,
    pub thread_id: Option<i64>,
    pub thread_title: Option<String>,
    pub reply_to_message_id: Option<i64>,
    pub reply_to_conversation_token: Option<String>,
    pub reply_to_conversation_name: Option<String>,
    pub reply_to_actor_display_name: Option<String>,
}

/// A rich object referenced from a message placeholder (user mention, file, call, ...).
/// All values are strings in the API.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct RichObjectParameter {
    #[serde(rename = "type")]
    pub object_type: String,
    pub id: String,
    pub name: String,
    pub server: Option<String>,
    pub link: Option<String>,
    pub call_type: Option<String>,
    pub icon_url: Option<String>,
    pub message_id: Option<String>,
    pub boardname: Option<String>,
    pub stackname: Option<String>,
    pub size: Option<String>,
    pub path: Option<String>,
    pub mimetype: Option<String>,
    pub preview_available: Option<String>,
    pub hide_download: Option<String>,
    pub mtime: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub description: Option<String>,
    pub thumb: Option<String>,
    pub website: Option<String>,
    pub visibility: Option<String>,
    pub assignable: Option<String>,
    pub conversation: Option<String>,
    pub etag: Option<String>,
    pub permissions: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub blurhash: Option<String>,
}

/// Suggestion for completing an `@mention`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MentionSuggestion {
    pub id: String,
    pub label: String,
    /// `users`, `groups`, `calls` (for `@all`), `guests`, `federated_users`, ...
    pub source: String,
    /// Insert as `@"<mention_id>"` in the message text.
    pub mention_id: String,
    pub details: Option<String>,
    #[serde(flatten)]
    pub user_status: UserStatus,
}

/// A page of chat messages.
#[derive(Debug, Clone, Default)]
pub struct ChatPage {
    pub messages: Vec<ChatMessage>,
    /// Pass as `last_known_message_id` to continue in the same direction.
    pub last_given: Option<i64>,
    /// Newest message id read by every participant (absent with read privacy).
    pub last_common_read: Option<i64>,
}

/// Body for sending a chat message.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessage {
    pub message: String,
    /// Message id to reply to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<i64>,
    /// Client-generated id to recognise the message when it comes back from the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    /// Don't trigger notifications.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub silent: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<i64>,
    /// Start a new thread with this title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_title: Option<String>,
}

impl SendMessage {
    pub fn text(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ..Default::default()
        }
    }

    pub fn reply(message: impl Into<String>, reply_to: i64) -> Self {
        Self {
            message: message.into(),
            reply_to: Some(reply_to),
            ..Default::default()
        }
    }
}

/// Which direction to page in when loading messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChatDirection {
    /// Messages older than `last_known_message_id` (newest first).
    #[default]
    Past,
    /// Messages newer than `last_known_message_id` (oldest first).
    Future,
}

/// Query for loading chat messages without waiting for new ones.
#[derive(Debug, Clone)]
pub struct HistoryQuery {
    pub direction: ChatDirection,
    /// 0 starts from the newest (past) or the read marker (future).
    pub last_known_message_id: i64,
    pub include_last_known: bool,
    /// At most 200.
    pub limit: u32,
    pub thread_id: Option<i64>,
    pub set_read_marker: bool,
    pub mark_notifications_as_read: bool,
    pub no_status_update: bool,
}

impl Default for HistoryQuery {
    fn default() -> Self {
        Self {
            direction: ChatDirection::Past,
            last_known_message_id: 0,
            include_last_known: false,
            limit: 100,
            thread_id: None,
            set_read_marker: false,
            mark_notifications_as_read: false,
            no_status_update: false,
        }
    }
}

/// Raw query string of `GET chat/{token}`, shared by history loading and the feed.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiveMessagesQuery {
    #[serde(serialize_with = "bool_as_int")]
    pub look_into_future: bool,
    pub last_known_message_id: i64,
    #[serde(serialize_with = "bool_as_int")]
    pub include_last_known: bool,
    pub limit: u32,
    /// Long-poll timeout in seconds (max 60), only used when looking into the future.
    pub timeout: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_common_read_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<i64>,
    #[serde(serialize_with = "bool_as_int")]
    pub set_read_marker: bool,
    #[serde(serialize_with = "bool_as_int")]
    pub mark_notifications_as_read: bool,
    #[serde(serialize_with = "bool_as_int")]
    pub no_status_update: bool,
}

impl From<&HistoryQuery> for ReceiveMessagesQuery {
    fn from(q: &HistoryQuery) -> Self {
        Self {
            look_into_future: q.direction == ChatDirection::Future,
            last_known_message_id: q.last_known_message_id,
            include_last_known: q.include_last_known,
            limit: q.limit,
            // Return immediately: history loading must not long-poll.
            timeout: 0,
            last_common_read_id: None,
            thread_id: q.thread_id,
            set_read_marker: q.set_read_marker,
            mark_notifications_as_read: q.mark_notifications_as_read,
            no_status_update: q.no_status_update,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_message_with_parent_and_parameters() {
        let json = r#"{
            "id": 42, "token": "abc", "actorType": "users", "actorId": "alice",
            "actorDisplayName": "Alice", "timestamp": 1700000000,
            "message": "Hi {mention-user1}", "messageType": "comment", "systemMessage": "",
            "messageParameters": {"mention-user1": {"type": "user", "id": "bob", "name": "Bob"}},
            "reactions": [], "isReplyable": true, "markdown": true, "referenceId": "",
            "expirationTimestamp": 0,
            "parent": {"id": 40, "deleted": true}
        }"#;
        let m: ChatMessage = serde_json::from_str(json).unwrap();
        assert_eq!(m.id, 42);
        assert_eq!(m.message_parameters["mention-user1"].id, "bob");
        assert!(m.reactions.is_empty());
        let parent = m.parent.unwrap();
        assert_eq!(parent.id, 40);
        assert!(parent.deleted);
    }

    #[test]
    fn send_message_skips_unset_fields() {
        let body = serde_json::to_value(SendMessage::reply("yo", 3)).unwrap();
        assert_eq!(body, serde_json::json!({"message": "yo", "replyTo": 3}));
    }

    #[test]
    fn receive_query_uses_integer_booleans() {
        let q = ReceiveMessagesQuery::from(&HistoryQuery::default());
        let v = serde_json::to_value(q).unwrap();
        assert_eq!(v["lookIntoFuture"], 0);
        assert_eq!(v["setReadMarker"], 0);
        assert!(v.get("threadId").is_none());
    }
}
