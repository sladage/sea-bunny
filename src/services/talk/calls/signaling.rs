//! Signaling configuration, and the internal signaling transport used when no
//! external signaling server is configured. WebRTC itself is out of scope.

use std::time::Duration;

use eventful_rs::*;

use crate::{
    dto::talk::{SignalingMessage, SignalingSettings},
    services::{
        ncclient::NcError,
        talk::{ApiVersion, talk_path},
    },
};

/// The server holds internal signaling pulls open for up to 30 seconds.
const PULL_TIMEOUT: Duration = Duration::from_secs(45);

service! {
    pub struct SignalingService;
}

#[asynchronize(pub)]
impl SignalingService {
    /// Signaling server, credentials and STUN/TURN servers. Pass the conversation
    /// token to get settings for a (possibly federated) conversation.
    #[asynced]
    pub async fn settings(&self, token: Option<String>) -> Result<SignalingSettings, NcError> {
        Ok(self
            .client
            .get(&talk_path(ApiVersion::V3, "signaling/settings"))
            .query(&[("token", token.unwrap_or_default())])
            .send()
            .await?
            .data)
    }

    /// Wait for internal signaling messages of a conversation the session joined.
    #[asynced]
    pub async fn pull(&self, token: String) -> Result<Vec<SignalingMessage>, NcError> {
        Ok(self
            .client
            .get(&signaling_path(&token))
            .timeout(PULL_TIMEOUT)
            .send()
            .await?
            .data)
    }

    /// Send internal signaling messages. Each message is a JSON object with at least
    /// `fn` (the encoded payload) and `sessionId` (the recipient).
    #[asynced]
    pub async fn send(
        &self,
        token: String,
        messages: Vec<serde_json::Value>,
    ) -> Result<(), NcError> {
        let messages = serde_json::Value::Array(messages).to_string();
        self.client
            .post(&signaling_path(&token))
            .json(&serde_json::json!({ "messages": messages }))
            .send_discarding_data()
            .await?;
        Ok(())
    }
}

fn signaling_path(token: &str) -> String {
    talk_path(ApiVersion::V3, format!("signaling/{token}"))
}
