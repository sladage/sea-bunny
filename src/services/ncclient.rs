use std::{cell::RefCell, sync::Arc};

use anyhow::Result;
use eventful_rs::*;
use reqwest::StatusCode;
use tokio::task;
use url::Url;

declare_shard!(pub NCClientShard, runtime = tokio);
use_shard!(shard = NCClientShard);

use crate::dto::ncauth::{
    LoginFlowV2Credentials, LoginFlowV2Poll, LoginFlowV2PollRequest, LoginFlowV2StartResponse,
};

pub enum AuthState {
    NeedsAuthentication,
    Authenticated,
}

#[derive(Debug, Clone)]
pub enum OnLogin {
    Success,
    Failure(String),
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("failed to open browser")]
    FailedToOpenBrowser,

    #[error("failed to save credentials in keyring: {0}")]
    FailedToSaveCredentials(#[source] keyring::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("login flow has not completed yet")]
    Pending,

    #[error("login flow token expired or is invalid")]
    InvalidOrExpired,
}

#[events]
pub trait NCClientEvents {
    fn on_login(&self, result: OnLogin);
}

#[eventful(NCClientEvents)]
#[derive(Debug)]
pub struct NCClient {
    client: reqwest::Client,
    ncserver: Url,
    credentials: RefCell<Option<(String, String)>>, // (login_name, app_password)
}

#[asynchronize(pub)]
impl NCClient {
    pub async fn new(ncserver: Url) -> Result<ShardRcHandle<Self>> {
        Self::spawn(|| Self {
            client: reqwest::Client::new(),
            ncserver,
            credentials: RefCell::new(None),
            events: Default::default(),
        })
        .await
        .map_err(|e| e.into())
    }

    /// Authenticate the user. If credentials are already stored in the keyring, verify them.
    /// If not, initiate the Login Flow v2 process and open the login URL in the user's default browser.
    #[asynced]
    pub async fn authenticate(&self) -> Result<AuthState, LoginError> {
        // Try to load credentials from the keyring.
        if !self.load_credentials().await {
            self.request_auth().await?;
            return Ok(AuthState::NeedsAuthentication);
        }

        // we have credentials, but let's verify them by making a simple request to the Nextcloud server.
        match self.verify_credentials().await {
            Ok(true) => Ok(AuthState::Authenticated),
            Ok(false) => {
                // Credentials are invalid; clear them and request authentication.
                self.credentials.replace(None);
                self.request_auth().await?;
                Ok(AuthState::NeedsAuthentication)
            }
            Err(e) => Err(e),
        }
    }

    async fn request_auth(&self) -> Result<(), LoginError> {
        let auth_token = self.get_auth_token().await?;

        // Open the login URL in the user's default browser.
        open::that(auth_token.login.as_str()).map_err(|_| LoginError::FailedToOpenBrowser)?;

        let self_handle = self.as_rc_handle().unwrap();
        tokio::spawn(async move {
            let client = self_handle;
            loop {
                let auth_token = auth_token.clone();
                // Check if there's an active auth request token.

                if !client.check_auth(auth_token.clone()).await {
                    break;
                }

                // Sleep for a while before the next iteration.
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        });

        Ok(())
    }

    #[asynced]
    async fn check_auth(&self, auth_token: LoginFlowV2StartResponse) -> bool {
        match self.poll_auth(&auth_token.poll).await {
            Ok(credentials) => {
                // Save the credentials in the keyring.
                if let Err(e) = self.save_credentials(credentials).await {
                    self.events.on_login().emit(OnLogin::Failure(e.to_string()));
                } else {
                    // Login successful.
                    self.events.on_login().emit(OnLogin::Success);
                }
                false
            }
            Err(LoginError::Pending) => {
                // Login flow is still pending; continue polling.
                true
            }
            Err(e) => {
                // An error occurred during polling.
                self.events.on_login().emit(OnLogin::Failure(e.to_string()));
                false
            }
        }
    }

    async fn nc_request_builder(&self, url: &str) -> Result<reqwest::RequestBuilder, LoginError> {
        let credentials = self.credentials.borrow();
        let (username, password) = match &*credentials {
            Some((u, p)) => (u.clone(), p.clone()),
            None => {
                return Err(LoginError::Pending);
            }
        };
        let builder = self
            .client
            .get(self.ncserver.join(url).unwrap())
            .basic_auth(username, Some(password))
            .header("Accept", "application/json")
            .header("User-Agent", "sea-bunny/0.1")
            .header("OCS-APIRequest", "true");
        Ok(builder)
    }

    /// Make a GET request to the Nextcloud server and deserialize the JSON response into the specified type.
    pub async fn get<T>(&self, url: &str) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self.nc_request_builder(url).await?.send().await?;
        let body = response.json::<T>().await?;
        Ok(body)
    }

    async fn verify_credentials(&self) -> Result<bool, LoginError> {
        let response = self
            .nc_request_builder("ocs/v2.php/cloud/user")
            .await?
            .send()
            .await?;
        match response.status() {
            StatusCode::OK => Ok(true),
            StatusCode::UNAUTHORIZED => Ok(false),
            _ => {
                response.error_for_status()?;
                unreachable!()
            }
        }
    }

    async fn get_auth_token(&self) -> Result<LoginFlowV2StartResponse, LoginError> {
        let url = self
            .ncserver
            .join("index.php/login/v2")
            .expect("valid login endpoint");

        let response = self.client.post(url).send().await?.error_for_status()?;

        Ok(response.json().await?)
    }

    async fn poll_auth(
        &self,
        poll: &LoginFlowV2Poll,
    ) -> Result<LoginFlowV2Credentials, LoginError> {
        let response = self
            .client
            .post(poll.endpoint.clone())
            .form(&LoginFlowV2PollRequest { token: &poll.token })
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),

            // Nextcloud deliberately returns 404 while the flow
            // hasn't completed yet.
            StatusCode::NOT_FOUND => Err(LoginError::Pending),

            _ => {
                response.error_for_status()?;
                unreachable!()
            }
        }
    }

    async fn load_credentials(&self) -> bool {
        let keyring = match keyring::Entry::new("sea-bunny", "nextcloud") {
            Ok(k) => k,
            Err(_) => return false,
        };

        let password = match keyring.get_password() {
            Ok(p) => p,
            Err(_) => return false,
        };

        let parts: Vec<&str> = password.splitn(2, ':').collect();
        if parts.len() != 2 {
            return false;
        }

        let login_name = parts[0].to_string();
        let app_password = parts[1].to_string();

        self.credentials.replace(Some((login_name, app_password)));

        true
    }

    async fn save_credentials(
        &self,
        credentials: LoginFlowV2Credentials,
    ) -> Result<(), LoginError> {
        // save in keyring
        let keyring = keyring::Entry::new("sea-bunny", "nextcloud")
            .map_err(LoginError::FailedToSaveCredentials)?;
        let password = format!("{}:{}", credentials.login_name, credentials.app_password);
        keyring
            .set_password(&password)
            .map_err(LoginError::FailedToSaveCredentials)?;
        self.credentials.replace(Some((
            credentials.login_name.clone(),
            credentials.app_password.clone(),
        )));
        Ok(())
    }
}
