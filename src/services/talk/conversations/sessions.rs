//! Joining and leaving a conversation as an active session.
//!
//! A session is required for calls and signaling, and an active session suppresses
//! notifications for the conversation while the user is looking at it.

use eventful_rs::*;

use crate::{
    dto::talk::{Conversation, SessionState},
    services::{ncclient::NcError, talk::room_path},
};

service! {
    pub struct ConversationSessionService;
}

#[asynchronize(pub)]
impl ConversationSessionService {
    /// Join the conversation, creating a session. The returned conversation carries
    /// the new `session_id`.
    ///
    /// With `force == false` and an existing session in another client, fails with
    /// HTTP 409 instead of taking over. A wrong password fails with HTTP 403 and
    /// error `password`.
    #[asynced]
    pub async fn join(
        &self,
        token: String,
        password: Option<String>,
        force: bool,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .post(&room_path(&token, "/participants/active"))
            .json(&serde_json::json!({
                "password": password.unwrap_or_default(),
                "force": force,
            }))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn leave(&self, token: String) -> Result<(), NcError> {
        self.client
            .delete(&room_path(&token, "/participants/active"))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Mark the session inactive (e.g. window hidden) so notifications are sent again.
    #[asynced]
    pub async fn set_state(
        &self,
        token: String,
        state: SessionState,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .put(&room_path(&token, "/participants/state"))
            .json(&serde_json::json!({ "state": state }))
            .send()
            .await?
            .data)
    }
}
