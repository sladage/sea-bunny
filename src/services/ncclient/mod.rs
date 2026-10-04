//! Authenticated transport to the Nextcloud server.
//!
//! `NCClient` owns the HTTP client, the credentials and the server capabilities.
//! It knows nothing about Talk; API areas are implemented as services in
//! [`crate::services::talk`] that live on the same shard and call the shard-local
//! request API in [`request`].

mod auth;
mod error;
mod request;

use std::cell::RefCell;

use eventful_rs::*;
use url::Url;

pub use error::NcError;
pub use request::{Download, NcRequest, OcsResponse};

use crate::dto::{capabilities::ServerCapabilities, user::CurrentUser};
use auth::Credentials;

declare_shard!(pub NCClientShard, runtime = tokio);
use_shard!(shard = NCClientShard);

pub enum AuthState {
    NeedsAuthentication,
    Authenticated,
}

#[derive(Debug, Clone)]
pub enum OnLogin {
    Success,
    Failure(String),
}

#[events]
pub trait NCClientEvents {
    /// Outcome of a browser login started by `authenticate`.
    fn on_login(&self, result: OnLogin);
    /// The server rejected the credentials of an authenticated session. Call
    /// `authenticate` again to log in.
    fn on_session_expired(&self) {}
    /// Talk capabilities or settings changed on the server; the cache was cleared.
    fn on_capabilities_changed(&self) {}
}

#[eventful(NCClientEvents)]
#[derive(Debug)]
pub struct NCClient {
    http: reqwest::Client,
    /// Always ends with `/` so relative paths join below it.
    server: Url,
    credentials: RefCell<Option<Credentials>>,
    user: RefCell<Option<CurrentUser>>,
    capabilities: RefCell<Option<ServerCapabilities>>,
    talk_hash: RefCell<Option<String>>,
    login_flow: RefCell<Option<TaskHandle>>,
}

#[asynchronize(pub)]
impl NCClient {
    pub async fn new(server: Url) -> Result<ShardRcHandle<Self>, NcError> {
        let server = normalize_server_url(server);
        Ok(Self::spawn(|| Self {
            http: reqwest::Client::new(),
            server,
            credentials: RefCell::new(None),
            user: RefCell::new(None),
            capabilities: RefCell::new(None),
            talk_hash: RefCell::new(None),
            login_flow: RefCell::new(None),
            events: Default::default(),
        })
        .await?)
    }

    /// Verify stored credentials, or start a browser login if there are none or they
    /// were rejected. A browser login reports its outcome through `on_login`.
    #[asynced]
    pub async fn authenticate(&self) -> Result<AuthState, NcError> {
        if self.load_credentials() {
            match self.fetch_current_user().await {
                Ok(_) => return Ok(AuthState::Authenticated),
                Err(NcError::Unauthorized) => {}
                Err(e) => return Err(e),
            }
        }
        self.start_login_flow().await?;
        Ok(AuthState::NeedsAuthentication)
    }

    /// Revoke the app password on the server and remove it from the keyring.
    #[asynced]
    pub async fn logout(&self) -> Result<(), NcError> {
        self.revoke_credentials().await
    }

    #[asynced]
    pub fn current_user(&self) -> Option<CurrentUser> {
        self.user.borrow().clone()
    }

    #[asynced]
    pub fn server_url(&self) -> Url {
        self.server.clone()
    }

    /// Server capabilities, fetched once and cached until Talk reports a change.
    #[asynced]
    pub async fn capabilities(&self) -> Result<ServerCapabilities, NcError> {
        if let Some(capabilities) = self.capabilities.borrow().clone() {
            return Ok(capabilities);
        }
        self.refresh_capabilities().await
    }

    #[asynced]
    pub async fn refresh_capabilities(&self) -> Result<ServerCapabilities, NcError> {
        let capabilities = self
            .get("ocs/v2.php/cloud/capabilities")
            .send::<ServerCapabilities>()
            .await?
            .data;
        self.capabilities.replace(Some(capabilities.clone()));
        Ok(capabilities)
    }

    #[asynced]
    pub async fn has_talk_feature(&self, feature: String) -> Result<bool, NcError> {
        Ok(self.capabilities().await?.has_talk_feature(&feature))
    }

    /// Id of the authenticated user (shard-local).
    pub fn user_id(&self) -> Option<String> {
        self.user.borrow().as_ref().map(|user| user.id.clone())
    }

    /// WebDAV path of a file in the user's storage, with every segment encoded.
    pub fn dav_file_path(&self, path: &str) -> Result<String, NcError> {
        let user = self.user_id().ok_or(NcError::NotAuthenticated)?;
        let encode = |segment: &str| {
            percent_encoding::utf8_percent_encode(segment, PATH_SEGMENT).to_string()
        };
        let segments: Vec<String> = path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(encode)
            .collect();
        Ok(format!(
            "remote.php/dav/files/{}/{}",
            encode(&user),
            segments.join("/")
        ))
    }

    fn observe_talk_hash(&self, hash: &str) {
        let previous = self.talk_hash.replace(Some(hash.to_owned()));
        if previous.is_some_and(|previous| previous != hash) {
            self.capabilities.replace(None);
            self.events.on_capabilities_changed().emit();
        }
    }
}

/// Everything except RFC 3986 unreserved characters.
const PATH_SEGMENT: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

/// `Url::join` replaces the last path segment unless the base ends with `/`,
/// which would break servers installed below a sub path.
fn normalize_server_url(mut url: Url) -> Url {
    if !url.path().ends_with('/') {
        let path = format!("{}/", url.path());
        url.set_path(&path);
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_url_keeps_sub_path() {
        let url = normalize_server_url(Url::parse("https://example.com/nextcloud").unwrap());
        assert_eq!(
            url.join("ocs/v2.php/cloud/user").unwrap().as_str(),
            "https://example.com/nextcloud/ocs/v2.php/cloud/user"
        );
        let url = normalize_server_url(Url::parse("https://example.com").unwrap());
        assert_eq!(url.as_str(), "https://example.com/");
    }
}
