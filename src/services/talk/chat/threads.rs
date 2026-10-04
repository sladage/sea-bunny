//! Threads: replies grouped under a message. Load thread messages with
//! `ChatService::history` and `HistoryQuery::thread_id`.

use eventful_rs::*;

use crate::{
    dto::talk::{NotificationLevel, ThreadInfo},
    services::{
        ncclient::NcError,
        talk::{ApiVersion, chat_path, talk_path},
    },
};

service! {
    pub struct ThreadService;
}

#[asynchronize(pub)]
impl ThreadService {
    /// Threads of a conversation, most recently active first.
    #[asynced]
    pub async fn recent(&self, token: String, limit: u32) -> Result<Vec<ThreadInfo>, NcError> {
        Ok(self
            .client
            .get(&chat_path(&token, "/threads/recent"))
            .query(&[("limit", limit)])
            .send()
            .await?
            .data)
    }

    /// Threads the user follows, across all conversations.
    #[asynced]
    pub async fn subscribed(&self, limit: u32, offset: u32) -> Result<Vec<ThreadInfo>, NcError> {
        Ok(self
            .client
            .get(&talk_path(ApiVersion::V1, "chat/subscribed-threads"))
            .query(&[("limit", limit), ("offset", offset)])
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn get(&self, token: String, thread_id: i64) -> Result<ThreadInfo, NcError> {
        Ok(self
            .client
            .get(&chat_path(&token, &format!("/threads/{thread_id}")))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn rename(
        &self,
        token: String,
        thread_id: i64,
        title: String,
    ) -> Result<ThreadInfo, NcError> {
        Ok(self
            .client
            .put(&chat_path(&token, &format!("/threads/{thread_id}")))
            .json(&serde_json::json!({ "threadTitle": title }))
            .send()
            .await?
            .data)
    }

    /// Notification level for a thread, identified by any of its messages.
    #[asynced]
    pub async fn set_notification_level(
        &self,
        token: String,
        message_id: i64,
        level: NotificationLevel,
    ) -> Result<ThreadInfo, NcError> {
        Ok(self
            .client
            .post(&chat_path(&token, &format!("/threads/{message_id}/notify")))
            .json(&serde_json::json!({ "level": level }))
            .send()
            .await?
            .data)
    }
}
