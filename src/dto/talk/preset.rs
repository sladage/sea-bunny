use std::collections::HashMap;

use serde::Deserialize;

use crate::dto::serde_ext::empty_as_default;

/// A server-defined set of conversation settings for creating conversations.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ConversationPreset {
    /// e.g. `default`, `webinar`, `presentation`, `hallway`.
    pub identifier: String,
    /// Translated name.
    pub name: String,
    pub description: String,
    /// Settings the preset applies, keyed by create parameter name.
    #[serde(deserialize_with = "empty_as_default")]
    pub parameters: HashMap<String, i64>,
}
