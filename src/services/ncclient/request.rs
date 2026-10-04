//! Request building and OCS response handling.
//!
//! These methods are shard-local (not dispatched through handles) because they are
//! generic over the response type. Services living on [`super::NCClientShard`] use
//! them through a local `ShardRc<NCClient>`.

use std::time::Duration;

use reqwest::{Method, StatusCode, header::HeaderMap};
use serde::{Serialize, de::DeserializeOwned};

use super::{NCClient, NcError};
use crate::dto::ocs::{OcsEnvelope, OcsErrorData};

const USER_AGENT: &str = concat!("sea-bunny/", env!("CARGO_PKG_VERSION"));

/// Header Talk uses to signal that capabilities or settings changed.
const TALK_HASH_HEADER: &str = "X-Nextcloud-Talk-Hash";

/// Unwrapped OCS response.
#[derive(Debug, Clone)]
pub struct OcsResponse<T> {
    pub data: T,
    pub status: StatusCode,
    pub headers: HeaderMap,
}

impl<T> OcsResponse<T> {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    pub fn header_i64(&self, name: &str) -> Option<i64> {
        self.header(name).and_then(|v| v.trim().parse().ok())
    }
}

/// A request against the Nextcloud server, authenticated as the current user.
#[must_use = "requests do nothing until sent"]
pub struct NcRequest<'a> {
    client: &'a NCClient,
    endpoint: String,
    builder: Result<reqwest::RequestBuilder, NcError>,
}

impl NCClient {
    /// Start a request to `path`, relative to the server URL (e.g. `ocs/v2.php/cloud/user`).
    pub fn request(&self, method: Method, path: &str) -> NcRequest<'_> {
        NcRequest {
            client: self,
            endpoint: format!("{method} {path}"),
            builder: self.build_request(method, path),
        }
    }

    pub fn get(&self, path: &str) -> NcRequest<'_> {
        self.request(Method::GET, path)
    }

    pub fn post(&self, path: &str) -> NcRequest<'_> {
        self.request(Method::POST, path)
    }

    pub fn put(&self, path: &str) -> NcRequest<'_> {
        self.request(Method::PUT, path)
    }

    pub fn delete(&self, path: &str) -> NcRequest<'_> {
        self.request(Method::DELETE, path)
    }

    fn build_request(
        &self,
        method: Method,
        path: &str,
    ) -> Result<reqwest::RequestBuilder, NcError> {
        let credentials = self.credentials.borrow();
        let credentials = credentials.as_ref().ok_or(NcError::NotAuthenticated)?;
        let url = self
            .server
            .join(path)
            .map_err(|e| NcError::InvalidUrl(format!("{path}: {e}")))?;
        Ok(self
            .unauthenticated(method, url)
            .basic_auth(&credentials.login_name, Some(&credentials.app_password)))
    }

    /// A request with the standard headers but without credentials (login flow).
    pub(super) fn unauthenticated(&self, method: Method, url: url::Url) -> reqwest::RequestBuilder {
        self.http
            .request(method, url)
            .header("Accept", "application/json")
            .header("User-Agent", USER_AGENT)
            .header("OCS-APIRequest", "true")
    }

    /// Inspect headers present on every response.
    fn observe_response(&self, response: &reqwest::Response) {
        if response.status() == StatusCode::UNAUTHORIZED {
            self.session_rejected();
        }
        if let Some(hash) = response
            .headers()
            .get(TALK_HASH_HEADER)
            .and_then(|v| v.to_str().ok())
        {
            self.observe_talk_hash(hash);
        }
    }
}

impl NcRequest<'_> {
    pub fn query<Q: Serialize + ?Sized>(mut self, query: &Q) -> Self {
        self.builder = self.builder.map(|b| b.query(query));
        self
    }

    pub fn json<B: Serialize + ?Sized>(mut self, body: &B) -> Self {
        self.builder = self.builder.map(|b| b.json(body));
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.builder = self.builder.map(|b| b.timeout(timeout));
        self
    }

    /// Send and deserialize the OCS `data`.
    pub async fn send<T: DeserializeOwned>(self) -> Result<OcsResponse<T>, NcError> {
        let endpoint = self.endpoint.clone();
        let response = self.execute().await?;
        decode(endpoint, response).await
    }

    /// Like [`Self::send`], but yields `None` on `304 Not Modified`.
    pub async fn send_unless_not_modified<T: DeserializeOwned>(
        self,
    ) -> Result<Option<OcsResponse<T>>, NcError> {
        let endpoint = self.endpoint.clone();
        let response = self.execute().await?;
        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(None);
        }
        decode(endpoint, response).await.map(Some)
    }

    /// Send and ignore the OCS `data`.
    pub async fn send_discarding_data(self) -> Result<OcsResponse<()>, NcError> {
        let response = self.execute().await?;
        Ok(OcsResponse {
            data: (),
            status: response.status(),
            headers: response.headers().clone(),
        })
    }

    /// Send, returning the raw response for non-OCS endpoints (avatars, files, ...).
    /// Error statuses are still mapped to [`NcError`].
    pub async fn send_raw(self) -> Result<reqwest::Response, NcError> {
        self.execute().await
    }

    async fn execute(self) -> Result<reqwest::Response, NcError> {
        let response = self.builder?.send().await?;
        self.client.observe_response(&response);

        let status = response.status();
        if status.is_success() || status == StatusCode::NOT_MODIFIED {
            return Ok(response);
        }
        if status == StatusCode::UNAUTHORIZED {
            return Err(NcError::Unauthorized);
        }

        let body = response.bytes().await.unwrap_or_default();
        let data = serde_json::from_slice::<OcsEnvelope<serde_json::Value>>(&body)
            .ok()
            .map(|env| {
                (
                    serde_json::from_value::<OcsErrorData>(env.ocs.data).ok(),
                    env.ocs.meta,
                )
            });
        let (error, message) = match data {
            Some((data, meta)) => {
                let data = data.unwrap_or_default();
                (data.error, data.message.or(meta.message))
            }
            None => (None, None),
        };
        Err(NcError::Api {
            endpoint: self.endpoint,
            status: status.as_u16(),
            error,
            message,
        })
    }
}

async fn decode<T: DeserializeOwned>(
    endpoint: String,
    response: reqwest::Response,
) -> Result<OcsResponse<T>, NcError> {
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.bytes().await?;
    let envelope: OcsEnvelope<T> =
        serde_json::from_slice(&body).map_err(|source| NcError::Decode {
            endpoint,
            source: source.into(),
        })?;
    Ok(OcsResponse {
        data: envelope.ocs.data,
        status,
        headers,
    })
}
