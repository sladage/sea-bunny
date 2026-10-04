use serde::Deserialize;

use super::FederationInviteState;

/// An invitation to a conversation on another server.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct FederationInvite {
    pub id: i64,
    pub state: FederationInviteState,
    pub room_name: String,
    pub inviter_cloud_id: String,
    pub inviter_display_name: String,
    pub remote_server_url: String,
    pub remote_token: String,
    pub remote_attendee_id: i64,
    pub local_cloud_id: String,
    /// Token of the local proxy conversation, once accepted.
    pub local_token: String,
    pub user_id: String,
}
