//! "Remind me later" for chat messages.

use eventful_rs::*;

use crate::{
    dto::talk::{ChatReminder, UpcomingReminder},
    services::{
        ncclient::NcError,
        talk::{ApiVersion, chat_path, talk_path},
    },
};

service! {
    pub struct ReminderService;
}

#[asynchronize(pub)]
impl ReminderService {
    /// Get a notification about the message at `timestamp` (Unix time).
    /// Replaces an existing reminder.
    #[asynced]
    pub async fn set(
        &self,
        token: String,
        message_id: i64,
        timestamp: i64,
    ) -> Result<ChatReminder, NcError> {
        Ok(self
            .client
            .post(&reminder_path(&token, message_id))
            .json(&serde_json::json!({ "timestamp": timestamp }))
            .send()
            .await?
            .data)
    }

    /// The reminder on a message; fails with HTTP 404 when there is none.
    #[asynced]
    pub async fn get(&self, token: String, message_id: i64) -> Result<ChatReminder, NcError> {
        Ok(self
            .client
            .get(&reminder_path(&token, message_id))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn delete(&self, token: String, message_id: i64) -> Result<(), NcError> {
        self.client
            .delete(&reminder_path(&token, message_id))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Upcoming reminders across all conversations.
    #[asynced]
    pub async fn upcoming(&self) -> Result<Vec<UpcomingReminder>, NcError> {
        Ok(self
            .client
            .get(&talk_path(ApiVersion::V1, "chat/upcoming-reminders"))
            .send()
            .await?
            .data)
    }
}

fn reminder_path(token: &str, message_id: i64) -> String {
    chat_path(token, &format!("/{message_id}/reminder"))
}
