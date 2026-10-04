//! Who can find and enter a conversation: guests, password, lobby, SIP dial-in
//! and read-only state. Moderators only.

use eventful_rs::*;

use crate::{
    dto::talk::{Conversation, ListableScope, LobbyState, ReadOnlyState, SipState},
    services::{ncclient::NcError, talk::room_path},
};

service! {
    pub struct ConversationAccessService;
}

#[asynchronize(pub)]
impl ConversationAccessService {
    /// Allow guests to join through a link, optionally protected by a password.
    #[asynced]
    pub async fn make_public(
        &self,
        token: String,
        password: Option<String>,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .post(&room_path(&token, "/public"))
            .json(&serde_json::json!({ "password": password.unwrap_or_default() }))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn make_private(&self, token: String) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .delete(&room_path(&token, "/public"))
            .send()
            .await?
            .data)
    }

    /// Set the guest password of a public conversation; empty removes it.
    #[asynced]
    pub async fn set_password(
        &self,
        token: String,
        password: String,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .put(&room_path(&token, "/password"))
            .json(&serde_json::json!({ "password": password }))
            .send()
            .await?
            .data)
    }

    /// Who can find and join the conversation without an invitation.
    #[asynced]
    pub async fn set_listable(
        &self,
        token: String,
        scope: ListableScope,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .put(&room_path(&token, "/listable"))
            .json(&serde_json::json!({ "scope": scope }))
            .send()
            .await?
            .data)
    }

    /// Enable or disable the lobby. `opens_at` is a Unix timestamp at which the
    /// lobby is disabled automatically.
    #[asynced]
    pub async fn set_lobby(
        &self,
        token: String,
        state: LobbyState,
        opens_at: Option<i64>,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .put(&room_path(&token, "/webinar/lobby"))
            .json(&serde_json::json!({ "state": state, "timer": opens_at }))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn set_sip(&self, token: String, state: SipState) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .put(&room_path(&token, "/webinar/sip"))
            .json(&serde_json::json!({ "state": state }))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn set_read_only(
        &self,
        token: String,
        state: ReadOnlyState,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .put(&room_path(&token, "/read-only"))
            .json(&serde_json::json!({ "state": state }))
            .send()
            .await?
            .data)
    }
}
