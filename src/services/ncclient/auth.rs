//! Login Flow v2, credential storage and verification.

use std::time::{Duration, Instant};

use eventful_rs::ShardRc;
use reqwest::{Method, StatusCode};

use super::{NCClient, NcError, OnLogin};
use crate::dto::{
    ncauth::{
        LoginFlowV2Credentials, LoginFlowV2Poll, LoginFlowV2PollRequest, LoginFlowV2StartResponse,
    },
    user::CurrentUser,
};

const KEYRING_SERVICE: &str = "sea-bunny";
const KEYRING_USER: &str = "nextcloud";

/// Nextcloud expires login flow tokens after 20 minutes.
const LOGIN_FLOW_LIFETIME: Duration = Duration::from_secs(20 * 60);
const LOGIN_POLL_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub(super) struct Credentials {
    pub login_name: String,
    pub app_password: String,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("login_name", &self.login_name)
            .finish_non_exhaustive()
    }
}

impl NCClient {
    /// Start Login Flow v2: open the login page and poll for completion in the background.
    /// The outcome is reported through `on_login`.
    pub(super) async fn start_login_flow(&self) -> Result<(), NcError> {
        let url = self
            .server
            .join("index.php/login/v2")
            .map_err(|e| NcError::InvalidUrl(e.to_string()))?;
        let flow: LoginFlowV2StartResponse = self
            .unauthenticated(Method::POST, url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        open::that(flow.login.as_str()).map_err(|e| NcError::Browser(e.to_string()))?;

        // Owned by the client: aborted when it is destroyed or on shard shutdown.
        let task = ShardRc::spawn_owned(&ShardRc::try_from_ref(self)?, |this| async move {
            let deadline = Instant::now() + LOGIN_FLOW_LIFETIME;
            loop {
                tokio::time::sleep(LOGIN_POLL_INTERVAL).await;
                let Some(client) = this.upgrade() else { return };
                if Instant::now() > deadline {
                    client.finish_login(Err(NcError::LoginFlowExpired));
                    return;
                }
                match client.poll_login_flow(&flow.poll).await {
                    Ok(None) => continue,
                    Ok(Some(credentials)) => {
                        let result = client.complete_login(credentials).await;
                        client.finish_login(result);
                        return;
                    }
                    Err(e) => {
                        client.finish_login(Err(e));
                        return;
                    }
                }
            }
        })?;

        if let Some(previous) = self.login_flow.replace(Some(task)) {
            previous.abort();
        }
        Ok(())
    }

    /// `Ok(None)` while the user has not completed the flow yet.
    async fn poll_login_flow(
        &self,
        poll: &LoginFlowV2Poll,
    ) -> Result<Option<LoginFlowV2Credentials>, NcError> {
        let response = self
            .unauthenticated(Method::POST, poll.endpoint.clone())
            .form(&LoginFlowV2PollRequest { token: &poll.token })
            .send()
            .await?;
        match response.status() {
            StatusCode::OK => Ok(Some(response.json().await?)),
            // Nextcloud deliberately answers 404 until the flow completes.
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(response.error_for_status().unwrap_err().into()),
        }
    }

    async fn complete_login(&self, credentials: LoginFlowV2Credentials) -> Result<(), NcError> {
        let credentials = Credentials {
            login_name: credentials.login_name,
            app_password: credentials.app_password,
        };
        save_credentials(&credentials)?;
        self.credentials.replace(Some(credentials));
        self.fetch_current_user().await?;
        Ok(())
    }

    fn finish_login(&self, result: Result<(), NcError>) {
        self.login_flow.replace(None);
        let event = match result {
            Ok(()) => OnLogin::Success,
            Err(e) => OnLogin::Failure(e.to_string()),
        };
        self.events.on_login().emit(event);
    }

    /// Log in without the login flow or keyring.
    #[cfg(test)]
    pub(crate) fn set_test_session(&self, login_name: &str, app_password: &str) {
        self.credentials.replace(Some(Credentials {
            login_name: login_name.to_owned(),
            app_password: app_password.to_owned(),
        }));
        self.user.replace(Some(CurrentUser {
            id: login_name.to_owned(),
            ..Default::default()
        }));
    }

    /// Load stored credentials into memory. Returns whether any were found.
    pub(super) fn load_credentials(&self) -> bool {
        let credentials = load_credentials();
        let found = credentials.is_some();
        self.credentials.replace(credentials);
        found
    }

    /// Fetch and cache the authenticated user. Fails with `Unauthorized` on bad credentials.
    pub(super) async fn fetch_current_user(&self) -> Result<CurrentUser, NcError> {
        let user = self
            .get("ocs/v2.php/cloud/user")
            .send::<CurrentUser>()
            .await?
            .data;
        self.user.replace(Some(user.clone()));
        Ok(user)
    }

    /// Revoke the app password on the server and forget all local credentials.
    pub(super) async fn revoke_credentials(&self) -> Result<(), NcError> {
        if let Some(flow) = self.login_flow.replace(None) {
            flow.abort();
        }
        // Forget locally even if the server can't be reached.
        let revoked = self
            .delete("ocs/v2.php/core/apppassword")
            .send_discarding_data()
            .await
            .map(drop);
        self.forget_session();
        delete_credentials()?;
        match revoked {
            Err(NcError::Unauthorized | NcError::NotAuthenticated) => Ok(()),
            other => other,
        }
    }

    pub(super) fn forget_session(&self) {
        self.credentials.replace(None);
        self.user.replace(None);
        self.capabilities.replace(None);
        self.talk_hash.replace(None);
    }

    /// Called when the server rejects our credentials.
    pub(super) fn session_rejected(&self) {
        let had_session = self.credentials.borrow().is_some() && self.user.borrow().is_some();
        self.forget_session();
        if had_session {
            self.events.on_session_expired().emit();
        }
    }
}

fn keyring_entry() -> Result<keyring::Entry, NcError> {
    Ok(keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?)
}

fn load_credentials() -> Option<Credentials> {
    let secret = keyring_entry().ok()?.get_password().ok()?;
    let (login_name, app_password) = secret.split_once(':')?;
    Some(Credentials {
        login_name: login_name.to_owned(),
        app_password: app_password.to_owned(),
    })
}

fn save_credentials(credentials: &Credentials) -> Result<(), NcError> {
    let secret = format!("{}:{}", credentials.login_name, credentials.app_password);
    Ok(keyring_entry()?.set_password(&secret)?)
}

fn delete_credentials() -> Result<(), NcError> {
    match keyring_entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
