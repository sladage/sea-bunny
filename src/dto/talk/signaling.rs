use serde::{Deserialize, Deserializer};

use super::{ActorType, InCallFlags, Permissions};

/// Settings for connecting to the signaling server and STUN/TURN servers.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SignalingSettings {
    /// `internal` or `external`.
    pub signaling_mode: String,
    /// External signaling server URL (empty in internal mode).
    pub server: String,
    pub ticket: Option<String>,
    pub user_id: Option<String>,
    pub hello_auth_params: Option<HelloAuthParams>,
    pub stunservers: Vec<IceServer>,
    pub turnservers: Vec<IceServer>,
    pub sip_dialin_info: String,
    pub hide_warning: bool,
    /// Present when the conversation is hosted on another server.
    pub federation: Option<SignalingFederationSettings>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct HelloAuthParams {
    #[serde(rename = "1.0")]
    pub v1: Option<HelloAuthV1>,
    #[serde(rename = "2.0")]
    pub v2: Option<HelloAuthV2>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct HelloAuthV1 {
    pub userid: Option<String>,
    pub ticket: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct HelloAuthV2 {
    /// JWT for the external signaling server's hello v2.
    pub token: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct IceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SignalingFederationSettings {
    pub server: String,
    pub nextcloud_server: String,
    pub room_id: String,
    pub hello_auth_params: HelloAuthV2,
}

/// A session in a conversation, as reported by internal signaling.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SignalingSession {
    pub session_id: String,
    pub actor_type: ActorType,
    pub actor_id: String,
    pub user_id: String,
    pub room_id: i64,
    pub in_call: InCallFlags,
    pub last_ping: i64,
    pub participant_permissions: Permissions,
}

/// A message received through internal signaling.
#[derive(Debug, Clone)]
pub enum SignalingMessage {
    /// Sessions currently in the conversation.
    UsersInRoom(Vec<SignalingSession>),
    /// A JSON-encoded message from another session (offer, answer, candidate, ...).
    Message(String),
    /// A message type this client does not interpret.
    Other {
        message_type: String,
        data: serde_json::Value,
    },
}

impl<'de> Deserialize<'de> for SignalingMessage {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(rename = "type")]
            message_type: String,
            #[serde(default)]
            data: serde_json::Value,
        }

        let raw = Raw::deserialize(deserializer)?;
        let parsed = match (raw.message_type.as_str(), raw.data) {
            ("usersInRoom", data) => {
                Self::UsersInRoom(serde_json::from_value(data).map_err(serde::de::Error::custom)?)
            }
            ("message", serde_json::Value::String(message)) => Self::Message(message),
            (_, data) => Self::Other {
                message_type: raw.message_type,
                data,
            },
        };
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_signaling_messages() {
        let json = r#"[
            {"type": "usersInRoom", "data": [{"sessionId": "s1", "inCall": 7, "roomId": 3}]},
            {"type": "message", "data": "{\"type\":\"offer\"}"},
            {"type": "control", "data": {"x": 1}}
        ]"#;
        let messages: Vec<SignalingMessage> = serde_json::from_str(json).unwrap();
        assert!(
            matches!(&messages[0], SignalingMessage::UsersInRoom(s) if s[0].session_id == "s1")
        );
        assert!(matches!(&messages[1], SignalingMessage::Message(m) if m.contains("offer")));
        assert!(
            matches!(&messages[2], SignalingMessage::Other { message_type, .. } if message_type == "control")
        );
    }
}
