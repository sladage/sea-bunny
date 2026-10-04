use std::sync::Arc;

use eventful_rs::InvokeError;

/// Error returned by every Nextcloud API call.
///
/// Cheap to clone so it can be forwarded in events and across shards.
#[derive(Debug, Clone, thiserror::Error)]
pub enum NcError {
    #[error("not authenticated")]
    NotAuthenticated,

    /// The server rejected the credentials (HTTP 401), e.g. the app password was revoked.
    #[error("the session expired or the credentials were revoked")]
    Unauthorized,

    #[error("HTTP error: {0}")]
    Http(Arc<reqwest::Error>),

    /// The server answered with a non-success status.
    #[error("{endpoint} failed with HTTP {status}{}", describe(.error, .message))]
    Api {
        endpoint: String,
        status: u16,
        /// Machine-readable error code from the OCS payload (e.g. `password`, `ban`).
        error: Option<String>,
        /// Human-readable message from the OCS payload.
        message: Option<String>,
    },

    #[error("failed to decode response of {endpoint}: {source}")]
    Decode {
        endpoint: String,
        source: Arc<serde_json::Error>,
    },

    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("keyring error: {0}")]
    Keyring(String),

    #[error("failed to open browser: {0}")]
    Browser(String),

    #[error("login flow expired before it was completed")]
    LoginFlowExpired,

    #[error("shard dispatch failed: {0}")]
    Dispatch(String),
}

impl NcError {
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Api { status, .. } => Some(*status),
            Self::Unauthorized => Some(401),
            Self::Http(e) => e.status().map(|s| s.as_u16()),
            _ => None,
        }
    }

    /// The OCS error code, for matching on documented Talk errors.
    pub fn api_error(&self) -> Option<&str> {
        match self {
            Self::Api { error, .. } => error.as_deref(),
            _ => None,
        }
    }

    /// Whether retrying the same request later may succeed.
    pub fn is_transient(&self) -> bool {
        match self {
            Self::Http(e) => !e.is_builder() && !e.is_decode(),
            Self::Api { status, .. } => *status == 429 || *status >= 500,
            _ => false,
        }
    }
}

fn describe(error: &Option<String>, message: &Option<String>) -> String {
    match (error, message) {
        (_, Some(message)) if !message.is_empty() => format!(": {message}"),
        (Some(error), _) if !error.is_empty() => format!(": {error}"),
        _ => String::new(),
    }
}

impl From<reqwest::Error> for NcError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(Arc::new(e))
    }
}

impl From<InvokeError> for NcError {
    fn from(e: InvokeError) -> Self {
        Self::Dispatch(e.to_string())
    }
}

impl From<keyring::Error> for NcError {
    fn from(e: keyring::Error) -> Self {
        Self::Keyring(e.to_string())
    }
}
