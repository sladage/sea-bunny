//! Bots installed on the server, enabled per conversation by moderators.

use eventful_rs::*;

use crate::{
    dto::talk::Bot,
    services::{
        ncclient::NcError,
        talk::{ApiVersion, talk_path},
    },
};

service! {
    pub struct BotService;
}

#[asynchronize(pub)]
impl BotService {
    /// Bots available to the conversation, with their state in it.
    #[asynced]
    pub async fn list(&self, token: String) -> Result<Vec<Bot>, NcError> {
        Ok(self.client.get(&bot_path(&token, "")).send().await?.data)
    }

    #[asynced]
    pub async fn enable(&self, token: String, bot_id: i64) -> Result<Option<Bot>, NcError> {
        Ok(self
            .client
            .post(&bot_path(&token, &format!("/{bot_id}")))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn disable(&self, token: String, bot_id: i64) -> Result<Option<Bot>, NcError> {
        Ok(self
            .client
            .delete(&bot_path(&token, &format!("/{bot_id}")))
            .send()
            .await?
            .data)
    }
}

fn bot_path(token: &str, suffix: &str) -> String {
    talk_path(ApiVersion::V1, format!("bot/{token}{suffix}"))
}
