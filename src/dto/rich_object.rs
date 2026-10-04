use serde::Deserialize;

/// A rich object referenced from a message placeholder (user mention, file, call, ...).
/// All values are strings in the API.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct RichObjectParameter {
    #[serde(rename = "type")]
    pub object_type: String,
    pub id: String,
    pub name: String,
    pub server: Option<String>,
    pub link: Option<String>,
    pub call_type: Option<String>,
    pub icon_url: Option<String>,
    pub message_id: Option<String>,
    pub boardname: Option<String>,
    pub stackname: Option<String>,
    pub size: Option<String>,
    pub path: Option<String>,
    pub mimetype: Option<String>,
    pub preview_available: Option<String>,
    pub hide_download: Option<String>,
    pub mtime: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub description: Option<String>,
    pub thumb: Option<String>,
    pub website: Option<String>,
    pub visibility: Option<String>,
    pub assignable: Option<String>,
    pub conversation: Option<String>,
    pub etag: Option<String>,
    pub permissions: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub blurhash: Option<String>,
}
