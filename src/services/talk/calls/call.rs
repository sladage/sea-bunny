//! Joining, leaving and managing calls. Media itself goes through WebRTC and the
//! signaling server (see `SignalingService`).

use eventful_rs::*;

use crate::{
    dto::talk::{CallNotificationState, CallPeer, InCallFlags, JoinCall},
    services::{
        ncclient::{Download, NcError},
        talk::{ApiVersion, talk_path},
    },
};

service! {
    pub struct CallService;
}

#[asynchronize(pub)]
impl CallService {
    /// Sessions currently in the call.
    #[asynced]
    pub async fn peers(&self, token: String) -> Result<Vec<CallPeer>, NcError> {
        Ok(self.client.get(&call_path(&token, "")).send().await?.data)
    }

    /// Join (or start) the call. Requires an active conversation session.
    #[asynced]
    pub async fn join(&self, token: String, options: JoinCall) -> Result<(), NcError> {
        self.client
            .post(&call_path(&token, ""))
            .json(&options)
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Change the media this session provides.
    #[asynced]
    pub async fn update_flags(&self, token: String, flags: InCallFlags) -> Result<(), NcError> {
        self.client
            .put(&call_path(&token, ""))
            .json(&serde_json::json!({ "flags": flags }))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Leave the call; with `end_for_everyone`, end it for all (moderators only).
    #[asynced]
    pub async fn leave(&self, token: String, end_for_everyone: bool) -> Result<(), NcError> {
        self.client
            .delete(&call_path(&token, ""))
            .query(&[("all", end_for_everyone)])
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Ring an attendee who is not in the call yet.
    #[asynced]
    pub async fn ring(&self, token: String, attendee_id: i64) -> Result<(), NcError> {
        self.client
            .post(&call_path(&token, &format!("/ring/{attendee_id}")))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Call a phone attendee through the SIP bridge.
    #[asynced]
    pub async fn dial_out(&self, token: String, attendee_id: i64) -> Result<(), NcError> {
        self.client
            .post(&call_path(&token, &format!("/dialout/{attendee_id}")))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Whether an incoming-call notification should keep ringing.
    #[asynced]
    pub async fn notification_state(
        &self,
        token: String,
    ) -> Result<CallNotificationState, NcError> {
        let result = self
            .client
            .get(&call_path(&token, "/notification-state"))
            .send_discarding_data()
            .await;
        match result {
            Ok(response) if response.status == reqwest::StatusCode::CREATED => {
                Ok(CallNotificationState::Missed)
            }
            Ok(_) => Ok(CallNotificationState::KeepRinging),
            Err(error) if error.status() == Some(404) => Ok(CallNotificationState::Dismiss),
            Err(error) => Err(error),
        }
    }

    /// Participants of the ongoing call as CSV (moderators only).
    #[asynced]
    pub async fn download_participants(&self, token: String) -> Result<Download, NcError> {
        self.client
            .get(&call_path(&token, "/download"))
            .query(&[("format", "csv")])
            .send_download()
            .await
    }
}

fn call_path(token: &str, suffix: &str) -> String {
    talk_path(ApiVersion::V4, format!("call/{token}{suffix}"))
}
