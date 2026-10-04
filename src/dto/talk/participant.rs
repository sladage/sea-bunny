use serde::Deserialize;

use super::{ActorType, InCallFlags, ParticipantType, Permissions, UserStatus};

/// An attendee of a conversation, with their active sessions.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Participant {
    pub attendee_id: i64,
    pub actor_type: ActorType,
    pub actor_id: String,
    pub invited_actor_id: Option<String>,
    pub display_name: String,
    pub participant_type: ParticipantType,
    pub room_token: String,
    pub permissions: Permissions,
    pub attendee_permissions: Permissions,
    pub attendee_pin: String,
    pub in_call: InCallFlags,
    pub last_ping: i64,
    pub session_ids: Vec<String>,
    pub actor_avatar_version: Option<String>,
    pub phone_number: Option<String>,
    pub call_id: Option<String>,
    #[serde(flatten)]
    pub user_status: UserStatus,
}

impl Participant {
    /// Whether the attendee currently has at least one session in the conversation.
    pub fn is_online(&self) -> bool {
        !self.session_ids.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_participant() {
        let json = r#"{
            "attendeeId": 12, "actorType": "users", "actorId": "bob", "displayName": "Bob",
            "participantType": 3, "inCall": 3, "sessionIds": ["s1"], "statusMessage": "busy"
        }"#;
        let p: Participant = serde_json::from_str(json).unwrap();
        assert_eq!(p.attendee_id, 12);
        assert_eq!(p.participant_type, ParticipantType::User);
        assert!(
            p.in_call
                .contains(InCallFlags::IN_CALL | InCallFlags::AUDIO)
        );
        assert!(p.is_online());
        assert_eq!(p.user_status.status_message.as_deref(), Some("busy"));
    }
}
