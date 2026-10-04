use serde::Deserialize;

use super::ConversationTagType;

/// A user-defined group for organising conversations.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ConversationTag {
    pub id: String,
    pub name: String,
    pub sort_order: i64,
    pub collapsed: bool,
    /// Built-in `favorites` and `other` tags can't be renamed or deleted.
    #[serde(rename = "type")]
    pub tag_type: ConversationTagType,
}
