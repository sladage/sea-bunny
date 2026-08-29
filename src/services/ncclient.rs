use std::sync::Arc;

use crate::events::{Event, EventHandle};
use event_derive::eventful;
use reqwest::StatusCode;
use tokio::task;
use url::Url;

use crate::{
    dto::ncauth::{
        LoginFlowV2Credentials, LoginFlowV2Poll, LoginFlowV2PollRequest, LoginFlowV2StartResponse,
    },
    models::config::config,
};

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

#[eventful(NCClient::internal)]
#[derive(Debug)]
struct NCClientInternal {
    client: reqwest::Client,
    ncserver: Url,
    auth_request_token: tokio::sync::Mutex<Option<LoginFlowV2StartResponse>>,
    credentials: tokio::sync::Mutex<Option<LoginFlowV2Credentials>>,

    #[event]
    on_login: OnLogin,
}

#[derive(Debug)]
pub struct NCClient {
    internal: Arc<NCClientInternal>,
    worker: task::JoinHandle<()>,
}

impl NCClient {
    pub fn new(ncserver: Url) -> Self {
        let nc_client = NCClientInternal {
            client: reqwest::Client::new(),
            ncserver,
            auth_request_token: tokio::sync::Mutex::new(None),
            credentials: tokio::sync::Mutex::new(None),
            on_login: Event::new(),
        };

        let internal = Arc::new(nc_client);
        let internal_worker = Arc::clone(&internal);
        let worker = task::spawn(async move {
            worker_loop(internal_worker).await;
        });

        Self { internal, worker }
    }

    // auth

    pub async fn request_auth(&self) -> Result<(), LoginError> {
        let auth_token = self.internal.get_auth_token().await?;

        // Open the login URL in the user's default browser.
        open::that(auth_token.login.as_str()).map_err(|_| LoginError::FailedToOpenBrowser)?;

        self.internal
            .auth_request_token
            .lock()
            .await
            .replace(auth_token);

        Ok(())
    }

    pub async fn get<T>(&self, url: &str) -> Result<T, reqwest::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self.internal.client.get(url).send().await?;
        let body = response.json::<T>().await?;
        Ok(body)
    }
}

impl Drop for NCClient {
    fn drop(&mut self) {
        self.worker.abort();
    }
}

impl NCClientInternal {
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
        self.credentials.lock().await.replace(credentials);
        Ok(())
    }
}

async fn worker_loop(internal: Arc<NCClientInternal>) {
    loop {
        // Check if there's an active auth request token.
        {
            let auth_request_token = {
                let lock = internal.auth_request_token.lock().await;
                lock.clone()
            };

            if let Some(auth_token) = auth_request_token {
                match internal.poll_auth(&auth_token.poll).await {
                    Ok(credentials) => {
                        // Save the credentials in the keyring.
                        if let Err(e) = internal.save_credentials(credentials).await {
                            internal.on_login.emit(OnLogin::Failure(format!(
                                "Failed to save credentials: {}",
                                e
                            )));
                        } else {
                            // Login successful.
                            internal.on_login.emit(OnLogin::Success);
                        }
                        // Clear the auth request token.
                        *internal.auth_request_token.lock().await = None;
                    }
                    Err(LoginError::Pending) => {
                        // Login flow is still pending; do nothing and wait for the next iteration.
                    }
                    Err(LoginError::InvalidOrExpired) => {
                        // Login flow has expired or is invalid; emit failure and clear the token.
                        internal
                            .on_login
                            .emit(OnLogin::Failure("Login flow expired".to_string()));
                        *internal.auth_request_token.lock().await = None;
                    }
                    Err(e) => {
                        // Other errors; emit failure and clear the token.
                        internal
                            .on_login
                            .emit(OnLogin::Failure(format!("Error: {}", e)));
                        *internal.auth_request_token.lock().await = None;
                    }
                }
            }
        }

        // Sleep for a while before the next iteration.
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}
