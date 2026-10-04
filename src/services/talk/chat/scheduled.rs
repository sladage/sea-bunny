//! Messages scheduled to be sent later. Only visible to their author.

use eventful_rs::*;

use crate::{
    dto::talk::{EditScheduledMessage, ScheduleMessage, ScheduledMessage},
    services::{ncclient::NcError, talk::chat_path},
};

service! {
    pub struct ScheduledMessageService;
}

#[asynchronize(pub)]
impl ScheduledMessageService {
    #[asynced]
    pub async fn list(&self, token: String) -> Result<Vec<ScheduledMessage>, NcError> {
        Ok(self
            .client
            .get(&chat_path(&token, "/schedule"))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn schedule(
        &self,
        token: String,
        message: ScheduleMessage,
    ) -> Result<ScheduledMessage, NcError> {
        Ok(self
            .client
            .post(&chat_path(&token, "/schedule"))
            .json(&message)
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn edit(
        &self,
        token: String,
        message_id: String,
        message: EditScheduledMessage,
    ) -> Result<ScheduledMessage, NcError> {
        Ok(self
            .client
            .post(&chat_path(&token, &format!("/schedule/{message_id}")))
            .json(&message)
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn delete(&self, token: String, message_id: String) -> Result<(), NcError> {
        self.client
            .delete(&chat_path(&token, &format!("/schedule/{message_id}")))
            .send_discarding_data()
            .await?;
        Ok(())
    }
}
