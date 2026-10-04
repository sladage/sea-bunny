use serde::Deserialize;

/// User status fields, embedded in conversations, participants and mention
/// suggestions when requested with `includeStatus`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UserStatus {
    pub status: Option<String>,
    pub status_icon: Option<String>,
    pub status_message: Option<String>,
    pub status_clear_at: Option<i64>,
}
