use serde::Deserialize;

/// Response from `GET ocs/v2.php/cloud/user`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CurrentUser {
    pub id: String,
    #[serde(alias = "display-name", alias = "displayname")]
    pub display_name: String,
    pub email: Option<String>,
    pub language: Option<String>,
    pub locale: Option<String>,
}
