use serde::{Deserialize, Serialize};

use super::{ActorType, InCallFlags};

/// A session currently in the call.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CallPeer {
    pub actor_type: ActorType,
    pub actor_id: String,
    pub display_name: String,
    pub session_id: String,
    pub token: String,
    pub last_ping: i64,
}

/// Body for joining a call.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinCall {
    /// Media the session provides; the server defaults to audio and video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<InCallFlags>,
    /// Don't notify the other participants.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub silent: bool,
    /// Consent to recording, required when the conversation demands it.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub recording_consent: bool,
    /// Participants of the previous call that should not be notified again.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub silent_for: Vec<String>,
}

/// What to do with an incoming-call notification, from `call/{token}/notification-state`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallNotificationState {
    /// The call is still ringing for this user.
    KeepRinging,
    /// The call ended without the user: show a "missed call" notification instead.
    Missed,
    /// The call ended or the user joined elsewhere: dismiss the notification.
    Dismiss,
}
