use serde::{Deserialize, Serialize};
use url::Url;

/// Response from:
///
/// POST {server}/index.php/login/v2
#[derive(Debug, Clone, Deserialize)]
pub struct LoginFlowV2StartResponse {
    /// URL the user should open in their browser.
    pub login: Url,

    /// Information needed to poll for completion.
    pub poll: LoginFlowV2Poll,
}

/// Polling information returned when starting Login Flow v2.
#[derive(Debug, Clone, Deserialize)]
pub struct LoginFlowV2Poll {
    /// Secret token used when polling.
    pub token: String,

    /// Absolute URL to POST the poll request to.
    ///
    /// Do not reconstruct this URL yourself; use the value returned
    /// by Nextcloud.
    pub endpoint: Url,
}

/// Form body sent to `LoginFlowV2Poll::endpoint`.
///
/// This is application/x-www-form-urlencoded, not JSON.
#[derive(Debug, Clone, Serialize)]
pub struct LoginFlowV2PollRequest<'a> {
    pub token: &'a str,
}

/// Successful response from the polling endpoint.
///
/// Returned once the user completed authentication and approved the client.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginFlowV2Credentials {
    /// Canonical/base URL of the Nextcloud instance.
    pub server: Url,

    /// Login name to use for HTTP Basic authentication.
    pub login_name: String,

    /// Generated application password.
    ///
    /// Treat this as a persistent secret.
    pub app_password: String,
}
