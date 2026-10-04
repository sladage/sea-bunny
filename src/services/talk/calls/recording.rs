//! Call recording through the recording server. Moderators only.

use eventful_rs::*;

use crate::{
    dto::talk::RecordingType,
    services::{
        ncclient::NcError,
        talk::{ApiVersion, talk_path},
    },
};

service! {
    pub struct RecordingService;
}

#[asynchronize(pub)]
impl RecordingService {
    #[asynced]
    pub async fn start(&self, token: String, recording_type: RecordingType) -> Result<(), NcError> {
        self.client
            .post(&recording_path(&token, ""))
            .json(&serde_json::json!({ "status": recording_type }))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    #[asynced]
    pub async fn stop(&self, token: String) -> Result<(), NcError> {
        self.client
            .delete(&recording_path(&token, ""))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Post a finished recording to the chat. `timestamp` identifies the
    /// "recording available" notification.
    #[asynced]
    pub async fn share_to_chat(
        &self,
        token: String,
        file_id: i64,
        timestamp: i64,
    ) -> Result<(), NcError> {
        self.client
            .post(&recording_path(&token, "/share-chat"))
            .json(&serde_json::json!({ "fileId": file_id, "timestamp": timestamp }))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Dismiss the "recording available" notification without sharing.
    #[asynced]
    pub async fn dismiss_notification(&self, token: String, timestamp: i64) -> Result<(), NcError> {
        self.client
            .delete(&recording_path(&token, "/notification"))
            .query(&[("timestamp", timestamp)])
            .send_discarding_data()
            .await?;
        Ok(())
    }
}

fn recording_path(token: &str, suffix: &str) -> String {
    talk_path(ApiVersion::V1, format!("recording/{token}{suffix}"))
}
