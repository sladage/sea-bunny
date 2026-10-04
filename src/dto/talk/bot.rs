use serde::Deserialize;

use super::BotState;

/// A bot installed on the server, with its state in one conversation.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Bot {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub state: BotState,
}
