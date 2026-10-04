//! Pinned messages. Pinning and unpinning require moderator permissions;
//! hiding a pinned message only affects the current user.

use eventful_rs::*;

use crate::{
    dto::talk::ChatMessage,
    services::{ncclient::NcError, talk::chat_path},
};

service! {
    pub struct PinService;
}

#[asynchronize(pub)]
impl PinService {
    /// Pin a message, until `until` (Unix time) or indefinitely. Returns the
    /// system message announcing it.
    #[asynced]
    pub async fn pin(
        &self,
        token: String,
        message_id: i64,
        until: Option<i64>,
    ) -> Result<Option<ChatMessage>, NcError> {
        Ok(self
            .client
            .post(&pin_path(&token, message_id, ""))
            .json(&serde_json::json!({ "pinUntil": until.unwrap_or(0) }))
            .send()
            .await?
            .data)
    }

    /// Returns the system message announcing it.
    #[asynced]
    pub async fn unpin(
        &self,
        token: String,
        message_id: i64,
    ) -> Result<Option<ChatMessage>, NcError> {
        Ok(self
            .client
            .delete(&pin_path(&token, message_id, ""))
            .send()
            .await?
            .data)
    }

    /// Hide the pinned message for the current user (see `Conversation::hidden_pinned_id`).
    #[asynced]
    pub async fn hide(&self, token: String, message_id: i64) -> Result<(), NcError> {
        self.client
            .delete(&pin_path(&token, message_id, "/self"))
            .send_discarding_data()
            .await?;
        Ok(())
    }
}

fn pin_path(token: &str, message_id: i64, suffix: &str) -> String {
    chat_path(token, &format!("/{message_id}/pin{suffix}"))
}
