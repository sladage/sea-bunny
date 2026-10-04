//! Talk constants, see <https://nextcloud-talk.readthedocs.io/en/latest/constants/>.
//!
//! Integer enums keep values unknown to this client in `Unknown(i64)` so a newer
//! server never breaks deserialization of a whole conversation list.

use serde::{Deserialize, Serialize};

int_enum! {
    pub enum ConversationType {
        OneToOne = 1,
        Group = 2,
        Public = 3,
        Changelog = 4,
        /// A one-to-one conversation whose other participant was deleted.
        FormerOneToOne = 5,
        NoteToSelf = 6,
    }
    default = Self::Unknown(0);
}

int_enum! {
    pub enum ParticipantType {
        Owner = 1,
        Moderator = 2,
        User = 3,
        Guest = 4,
        /// A user who joined through a public link.
        UserSelfJoined = 5,
        GuestModerator = 6,
    }
    default = Self::Unknown(0);
}

impl ParticipantType {
    pub fn is_moderator(self) -> bool {
        matches!(self, Self::Owner | Self::Moderator | Self::GuestModerator)
    }
}

int_enum! {
    pub enum ReadOnlyState {
        ReadWrite = 0,
        ReadOnly = 1,
    }
    default = Self::ReadWrite;
}

int_enum! {
    pub enum ListableScope {
        ParticipantsOnly = 0,
        /// Regular users, excluding users created with the Guests app.
        RegularUsers = 1,
        Everyone = 2,
    }
    default = Self::ParticipantsOnly;
}

int_enum! {
    pub enum LobbyState {
        NoLobby = 0,
        NonModerators = 1,
    }
    default = Self::NoLobby;
}

int_enum! {
    pub enum SipState {
        Disabled = 0,
        /// Each participant needs a unique PIN.
        Enabled = 1,
        /// Only the conversation token is required.
        EnabledNoPin = 2,
    }
    default = Self::Disabled;
}

int_enum! {
    pub enum BreakoutRoomMode {
        NotConfigured = 0,
        Automatic = 1,
        Manual = 2,
        Free = 3,
    }
    default = Self::NotConfigured;
}

int_enum! {
    pub enum BreakoutRoomStatus {
        Stopped = 0,
        Started = 1,
    }
    default = Self::Stopped;
}

int_enum! {
    /// Who may mention `@all`.
    pub enum MentionPermissions {
        Everyone = 0,
        Moderators = 1,
    }
    default = Self::Everyone;
}

int_enum! {
    pub enum NotificationLevel {
        /// Always for one-to-one conversations, on mention for others.
        Default = 0,
        Always = 1,
        Mention = 2,
        Never = 3,
    }
    default = Self::Default;
}

int_enum! {
    pub enum CallNotificationLevel {
        Off = 0,
        On = 1,
    }
    default = Self::On;
}

int_enum! {
    pub enum SessionState {
        /// Notifications are still sent while this session is in the conversation.
        Inactive = 0,
        /// No notifications are sent.
        Active = 1,
    }
    default = Self::Active;
}

int_enum! {
    pub enum CallRecordingStatus {
        None = 0,
        Video = 1,
        Audio = 2,
        StartingVideo = 3,
        StartingAudio = 4,
        Failed = 5,
    }
    default = Self::None;
}

int_enum! {
    pub enum RecordingConsent {
        NotRequired = 0,
        Required = 1,
        /// Moderators decide per conversation (server config level only).
        ModeratorChoice = 2,
    }
    default = Self::NotRequired;
}

int_flags! {
    /// Attendee permissions. An empty set means "default": inherit from the next level.
    pub struct Permissions: u32 {
        /// Required to be able to remove all other permissions.
        const CUSTOM = 1;
        const START_CALL = 2;
        const JOIN_CALL = 4;
        const IGNORE_LOBBY = 8;
        const PUBLISH_AUDIO = 16;
        const PUBLISH_VIDEO = 32;
        const PUBLISH_SCREEN = 64;
        /// Post chat messages and share items.
        const CHAT = 128;
        const REACT = 256;
    }
}

int_flags! {
    /// Participant in-call flags. Empty means disconnected.
    pub struct InCallFlags: u32 {
        const IN_CALL = 1;
        const AUDIO = 2;
        const VIDEO = 4;
        const SIP = 8;
    }
}

int_flags! {
    pub struct ConversationAttributes: u32 {
        /// Join the call when joining the conversation.
        const VOICE_ROOM = 1;
        /// Cannot be deleted, cleared or have its guest/listable settings changed.
        const PRESERVED = 2;
        const CLASSIFIED = 4;
        const CHANNEL = 8;
        const ANNOUNCEMENT = 16;
    }
}

/// Actor type of a participant or chat message author.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    Users,
    FederatedUsers,
    Groups,
    Circles,
    Teams,
    Guests,
    Emails,
    Phones,
    Bots,
    Bridged,
    DeletedUsers,
    #[default]
    #[serde(other)]
    Unknown,
}

/// What kind of actor to invite when adding a participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantSource {
    #[default]
    Users,
    Groups,
    Circles,
    Teams,
    Emails,
    FederatedUsers,
    Phones,
}

/// How a permission change is applied to the existing permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionMethod {
    Set,
    Add,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Deserialize)]
pub enum MessageType {
    #[default]
    #[serde(rename = "comment")]
    Comment,
    #[serde(rename = "comment_deleted")]
    CommentDeleted,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "command")]
    Command,
    #[serde(rename = "voice-message")]
    VoiceMessage,
    #[serde(rename = "record-audio")]
    RecordAudio,
    #[serde(rename = "record-video")]
    RecordVideo,
    #[serde(other)]
    Unknown,
}

int_enum! {
    /// Read-status and typing privacy of the current user.
    pub enum Privacy {
        Public = 0,
        Private = 1,
    }
    default = Self::Public;
}

int_enum! {
    pub enum PollStatus {
        Open = 0,
        Closed = 1,
        Draft = 2,
    }
    default = Self::Open;
}

int_enum! {
    pub enum PollResultMode {
        /// Results and voters are visible immediately.
        Public = 0,
        /// Only vote counts are shown, after the poll closes.
        Hidden = 1,
    }
    default = Self::Public;
}

int_enum! {
    pub enum BotState {
        Disabled = 0,
        Enabled = 1,
        /// Can neither be enabled nor disabled by a moderator.
        NoSetup = 2,
    }
    default = Self::Disabled;
}

int_enum! {
    pub enum FederationInviteState {
        Pending = 0,
        Accepted = 1,
    }
    default = Self::Pending;
}

int_enum! {
    pub enum RecordingType {
        Video = 1,
        Audio = 2,
    }
    default = Self::Video;
}

/// Actor kinds that can be banned from a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BanActorType {
    Users,
    Guests,
    Emails,
    Ip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConversationTagType {
    #[default]
    Custom,
    Favorites,
    #[serde(other)]
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_enum_roundtrip_and_unknown() {
        let t: ConversationType = serde_json::from_str("6").unwrap();
        assert_eq!(t, ConversationType::NoteToSelf);
        let t: ConversationType = serde_json::from_str("42").unwrap();
        assert_eq!(t, ConversationType::Unknown(42));
        assert_eq!(serde_json::to_string(&t).unwrap(), "42");
    }

    #[test]
    fn flags_serialize_as_integers_and_keep_unknown_bits() {
        let p: Permissions = serde_json::from_str("130").unwrap();
        assert!(p.contains(Permissions::CHAT | Permissions::START_CALL));
        let p: Permissions = serde_json::from_str("1024").unwrap();
        assert_eq!(serde_json::to_string(&p).unwrap(), "1024");
    }

    #[test]
    fn string_enums_fall_back_to_unknown() {
        let a: ActorType = serde_json::from_str(r#""federated_users""#).unwrap();
        assert_eq!(a, ActorType::FederatedUsers);
        let a: ActorType = serde_json::from_str(r#""martians""#).unwrap();
        assert_eq!(a, ActorType::Unknown);
        let m: MessageType = serde_json::from_str(r#""voice-message""#).unwrap();
        assert_eq!(m, MessageType::VoiceMessage);
    }
}
