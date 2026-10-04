use serde::{Deserialize, Serialize};

use super::{
    ActorType, BreakoutRoomMode, BreakoutRoomStatus, CallNotificationLevel, CallRecordingStatus,
    ChatMessage, ConversationAttributes, ConversationType, InCallFlags, ListableScope, LobbyState,
    MentionPermissions, NotificationLevel, ParticipantType, Permissions, ReadOnlyState,
    RecordingConsent, SipState, UserStatus,
};
use crate::dto::serde_ext::empty_array_as_none;

/// A Talk conversation ("room" in the API), as seen by the current user.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Conversation {
    pub id: i64,
    pub token: String,
    #[serde(rename = "type")]
    pub conversation_type: ConversationType,
    /// Internal name; for one-to-one conversations the other user's id.
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub object_type: String,
    pub object_id: String,
    pub attributes: ConversationAttributes,

    // The current user's attendee record in this conversation.
    pub actor_type: ActorType,
    pub actor_id: String,
    pub invited_actor_id: Option<String>,
    pub attendee_id: i64,
    pub attendee_pin: Option<String>,
    pub participant_type: ParticipantType,
    pub session_id: String,

    // Permissions.
    /// Effective permissions of the current user.
    pub permissions: Permissions,
    pub attendee_permissions: Permissions,
    pub call_permissions: Permissions,
    pub default_permissions: Permissions,
    pub can_start_call: bool,
    pub can_leave_conversation: bool,
    pub can_delete_conversation: bool,
    #[serde(rename = "canEnableSIP")]
    pub can_enable_sip: bool,

    // Settings.
    pub has_password: bool,
    pub read_only: ReadOnlyState,
    pub listable: ListableScope,
    pub lobby_state: LobbyState,
    /// Unix timestamp at which the lobby opens, 0 when not scheduled.
    pub lobby_timer: i64,
    pub sip_enabled: SipState,
    pub mention_permissions: MentionPermissions,
    /// Seconds after which messages expire, 0 when disabled.
    pub message_expiration: i64,
    pub recording_consent: RecordingConsent,
    pub breakout_room_mode: BreakoutRoomMode,
    pub breakout_room_status: BreakoutRoomStatus,
    pub live_transcription_language_id: String,

    // Per-user preferences.
    pub is_favorite: bool,
    pub is_archived: bool,
    pub is_important: bool,
    pub is_sensitive: bool,
    pub notification_level: NotificationLevel,
    pub notification_calls: CallNotificationLevel,
    pub tag_ids: Vec<String>,

    // Avatar.
    pub avatar_version: String,
    pub is_custom_avatar: bool,

    // Chat state.
    pub last_activity: i64,
    pub last_ping: i64,
    pub last_read_message: i64,
    pub last_common_read_message: i64,
    pub unread_messages: i64,
    pub unread_mention: bool,
    pub unread_mention_direct: bool,
    /// Not present for empty conversations. Never contains `parent` or `reactionsSelf`.
    #[serde(deserialize_with = "empty_array_as_none")]
    pub last_message: Option<ChatMessage>,
    pub last_pinned_id: i64,
    pub hidden_pinned_id: i64,
    pub has_scheduled_messages: i64,

    // Call state.
    pub has_call: bool,
    pub call_start_time: i64,
    /// Combined flags of all participants in the call.
    pub call_flag: InCallFlags,
    /// Flags of the session making the request.
    pub participant_flags: InCallFlags,
    pub call_recording: CallRecordingStatus,

    // Federation.
    pub remote_server: Option<String>,
    pub remote_token: Option<String>,

    /// Status of the other user in one-to-one conversations (with `includeStatus`).
    #[serde(flatten)]
    pub user_status: UserStatus,
}

impl Conversation {
    pub fn is_federated(&self) -> bool {
        self.remote_server.as_deref().is_some_and(|s| !s.is_empty())
    }
}

/// Result of listing conversations.
#[derive(Debug, Clone, Default)]
pub struct ConversationList {
    pub conversations: Vec<Conversation>,
    /// Pass as `modified_since` on the next call to only get changed conversations.
    pub modified_before: Option<i64>,
    /// Number of pending federation invitations.
    pub pending_federation_invites: Option<i64>,
}

/// Query for listing conversations.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationListQuery {
    /// Only return conversations modified since this timestamp (0 for all).
    pub modified_since: i64,
    pub include_status: bool,
    pub include_last_message: bool,
    /// Don't mark the user as online (use for background refreshes).
    #[serde(serialize_with = "crate::dto::serde_ext::bool_as_int")]
    pub no_status_update: bool,
}

impl Default for ConversationListQuery {
    fn default() -> Self {
        Self {
            modified_since: 0,
            include_status: false,
            include_last_message: true,
            no_status_update: false,
        }
    }
}

/// Actors to invite, grouped by type.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct InvitationList {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub federated_users: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub emails: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub phones: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub teams: Vec<String>,
}

impl InvitationList {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// Body for creating a conversation. Only set fields are sent.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConversation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_type: Option<ConversationType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "InvitationList::is_empty")]
    pub participants: InvitationList,
    /// Server-defined preset id, see the presets endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<ReadOnlyState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listable: Option<ListableScope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_expiration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lobby_state: Option<LobbyState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lobby_timer: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sip_enabled: Option<SipState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recording_consent: Option<RecordingConsent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention_permissions: Option<MentionPermissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_color: Option<String>,
}

impl CreateConversation {
    /// A one-to-one conversation with `user_id`. Returns the existing one if present.
    pub fn one_to_one(user_id: impl Into<String>) -> Self {
        Self {
            room_type: Some(ConversationType::OneToOne),
            participants: InvitationList {
                users: vec![user_id.into()],
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn group(name: impl Into<String>, participants: InvitationList) -> Self {
        Self {
            room_type: Some(ConversationType::Group),
            room_name: Some(name.into()),
            participants,
            ..Default::default()
        }
    }

    pub fn public(name: impl Into<String>, participants: InvitationList) -> Self {
        Self {
            room_type: Some(ConversationType::Public),
            room_name: Some(name.into()),
            participants,
            ..Default::default()
        }
    }
}

/// Result of creating a conversation.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CreatedConversation {
    #[serde(flatten)]
    pub conversation: Conversation,
    /// Invitations the server could not deliver (HTTP 202).
    #[serde(deserialize_with = "empty_array_as_none")]
    pub invalid_participants: Option<InvitationList>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_conversation_with_empty_last_message() {
        let json = r#"{
            "id": 7, "token": "abc123", "type": 2, "name": "team", "displayName": "Team",
            "actorType": "users", "actorId": "alice", "participantType": 1,
            "permissions": 254, "canEnableSIP": true, "lastMessage": [],
            "unreadMessages": 3, "status": "online", "attributes": 2
        }"#;
        let c: Conversation = serde_json::from_str(json).unwrap();
        assert_eq!(c.token, "abc123");
        assert_eq!(c.conversation_type, ConversationType::Group);
        assert!(c.participant_type.is_moderator());
        assert!(c.permissions.contains(Permissions::CHAT));
        assert!(c.can_enable_sip);
        assert!(c.last_message.is_none());
        assert_eq!(c.user_status.status.as_deref(), Some("online"));
        assert!(c.attributes.contains(ConversationAttributes::PRESERVED));
    }

    #[test]
    fn parses_conversation_with_last_message() {
        let json =
            r#"{"token": "t", "lastMessage": {"id": 5, "message": "hi", "messageParameters": []}}"#;
        let c: Conversation = serde_json::from_str(json).unwrap();
        assert_eq!(c.last_message.unwrap().id, 5);
    }

    #[test]
    fn create_body_only_contains_set_fields() {
        let body = serde_json::to_value(CreateConversation::one_to_one("bob")).unwrap();
        assert_eq!(
            body,
            serde_json::json!({"roomType": 1, "participants": {"users": ["bob"]}})
        );
    }
}
