use serde::Deserialize;

use super::ActorType;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Ban {
    pub id: i64,
    pub moderator_actor_type: ActorType,
    pub moderator_actor_id: String,
    pub moderator_display_name: String,
    /// `users`, `guests`, `emails` or `ip`.
    pub banned_actor_type: String,
    pub banned_actor_id: String,
    pub banned_display_name: String,
    pub banned_time: i64,
    /// Only visible to moderators.
    pub internal_note: String,
}
