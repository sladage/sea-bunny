use serde::Deserialize;

/// Every OCS endpoint wraps its payload as `{ "ocs": { "meta": ..., "data": ... } }`.
#[derive(Debug, Clone, Deserialize)]
pub struct OcsEnvelope<T> {
    pub ocs: OcsBody<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OcsBody<T> {
    pub meta: OcsMeta,
    pub data: T,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct OcsMeta {
    pub status: String,
    pub statuscode: i64,
    pub message: Option<String>,
}

/// Error payload returned by most Talk endpoints: `{ "error": "<code>", "message": "..." }`.
///
/// `data` may also be `null` or `[]` on errors, in which case both fields are `None`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct OcsErrorData {
    pub error: Option<String>,
    pub message: Option<String>,
}
