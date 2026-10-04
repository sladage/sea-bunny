use serde::Deserialize;
use serde_json::{Map, Value};

/// Matterbridge configuration of a conversation, with its process state.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Matterbridge {
    pub enabled: bool,
    /// One object per bridged service; fields depend on the service type.
    pub parts: Vec<Map<String, Value>>,
    pub pid: i64,
    pub running: bool,
    pub log: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MatterbridgeProcessState {
    pub running: bool,
    pub log: String,
}
