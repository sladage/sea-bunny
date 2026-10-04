//! Banning actors from a conversation. Moderators only.

use eventful_rs::*;

use crate::{
    dto::talk::{Ban, BanActorType},
    services::{
        ncclient::NcError,
        talk::{ApiVersion, talk_path},
    },
};

service! {
    pub struct BanService;
}

#[asynchronize(pub)]
impl BanService {
    #[asynced]
    pub async fn list(&self, token: String) -> Result<Vec<Ban>, NcError> {
        Ok(self.client.get(&ban_path(&token, "")).send().await?.data)
    }

    /// Ban an actor (removing them if present). `internal_note` is only shown to
    /// moderators.
    #[asynced]
    pub async fn ban(
        &self,
        token: String,
        actor_type: BanActorType,
        actor_id: String,
        internal_note: String,
    ) -> Result<Ban, NcError> {
        Ok(self
            .client
            .post(&ban_path(&token, ""))
            .json(&serde_json::json!({
                "actorType": actor_type,
                "actorId": actor_id,
                "internalNote": internal_note,
            }))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn unban(&self, token: String, ban_id: i64) -> Result<(), NcError> {
        self.client
            .delete(&ban_path(&token, &format!("/{ban_id}")))
            .send_discarding_data()
            .await?;
        Ok(())
    }
}

fn ban_path(token: &str, suffix: &str) -> String {
    talk_path(ApiVersion::V1, format!("ban/{token}{suffix}"))
}
