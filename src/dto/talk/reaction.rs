use std::collections::HashMap;

use serde::Deserialize;

use super::ActorType;

/// Who reacted, and when.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Reaction {
    pub actor_type: ActorType,
    pub actor_id: String,
    pub actor_display_name: String,
    pub timestamp: i64,
}

/// Reactions of a message, keyed by emoji.
pub type Reactions = HashMap<String, Vec<Reaction>>;
