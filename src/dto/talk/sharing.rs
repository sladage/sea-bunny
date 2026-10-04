use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ChatMessage;

/// Categories of items shared in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SharedItemType {
    Audio,
    Deckcard,
    File,
    Location,
    /// Images and videos.
    Media,
    Other,
    Poll,
    Recording,
    Voice,
}

/// A page of shared items of one type, newest first.
#[derive(Debug, Clone, Default)]
pub struct SharedItems {
    pub messages: Vec<ChatMessage>,
    /// Pass as `last_known_message_id` to load older items.
    pub last_given: Option<i64>,
}

/// The newest few shared items per type, keyed by type name (`media`, `file`, ...).
pub type SharedItemsOverview = HashMap<String, Vec<ChatMessage>>;

/// Body for sharing a rich object (e.g. a location or deck card) into a chat.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareObject {
    /// Rich object type, e.g. `geo-location`.
    pub object_type: String,
    pub object_id: String,
    /// JSON-encoded rich object data (must contain at least `type`, `id` and `name`).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub meta_data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<i64>,
}

/// Where uploads for a conversation go, and how colliding file names were renamed.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AttachmentFolder {
    /// Folder in the user's files, relative to their root.
    pub folder: String,
    /// One map per renamed file: original name to new name.
    pub renames: Vec<HashMap<String, String>>,
}

/// Message-related options for a shared file, sent as `talkMetaData`.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentMetaData {
    /// Text posted together with the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<i64>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub silent: bool,
    /// `voice-message` to post an audio file as a voice message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<i64>,
}

/// Body for posting a file from the user's storage to a conversation.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostAttachment {
    /// Path in the user's files.
    pub file_path: String,
    pub reference_id: String,
    /// JSON-encoded [`AttachmentMetaData`].
    #[serde(skip_serializing_if = "String::is_empty")]
    pub talk_meta_data: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub file_name: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub allow_update: bool,
}

/// The conversation of a public file share, and the share owner.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ShareConversation {
    pub token: String,
    pub user_id: String,
    pub user_display_name: String,
}

/// A conversation created to verify the recipient of a password-protected share.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ShareAuthConversation {
    pub token: String,
    pub name: String,
    pub display_name: String,
}
