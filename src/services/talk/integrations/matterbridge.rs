//! Bridging a conversation to other chat services through Matterbridge.
//! Moderators only, and only when `config => chat => matterbridge-enabled`.

use eventful_rs::*;
use serde_json::{Map, Value};

use crate::{
    dto::talk::{Matterbridge, MatterbridgeProcessState},
    services::{
        ncclient::NcError,
        talk::{ApiVersion, talk_path},
    },
};

service! {
    pub struct MatterbridgeService;
}

#[asynchronize(pub)]
impl MatterbridgeService {
    #[asynced]
    pub async fn get(&self, token: String) -> Result<Matterbridge, NcError> {
        Ok(self.client.get(&bridge_path(&token, "")).send().await?.data)
    }

    /// Replace the bridge configuration and start or stop the bridge process.
    #[asynced]
    pub async fn update(
        &self,
        token: String,
        enabled: bool,
        parts: Vec<Map<String, Value>>,
    ) -> Result<MatterbridgeProcessState, NcError> {
        Ok(self
            .client
            .put(&bridge_path(&token, ""))
            .json(&serde_json::json!({ "enabled": enabled, "parts": parts }))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn delete(&self, token: String) -> Result<(), NcError> {
        self.client
            .delete(&bridge_path(&token, ""))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    #[asynced]
    pub async fn process_state(&self, token: String) -> Result<MatterbridgeProcessState, NcError> {
        Ok(self
            .client
            .get(&bridge_path(&token, "/process"))
            .send()
            .await?
            .data)
    }
}

fn bridge_path(token: &str, suffix: &str) -> String {
    talk_path(ApiVersion::V1, format!("bridge/{token}{suffix}"))
}
