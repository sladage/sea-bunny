//! The current user's personal settings for a conversation: favorite, archive,
//! importance, sensitivity, notifications and tags.

use eventful_rs::*;

use crate::{
    dto::talk::{CallNotificationLevel, Conversation, NotificationLevel},
    services::{ncclient::NcError, talk::room_path},
};

service! {
    pub struct ConversationPreferenceService;
}

#[asynchronize(pub)]
impl ConversationPreferenceService {
    #[asynced]
    pub async fn set_favorite(
        &self,
        token: String,
        favorite: bool,
    ) -> Result<Conversation, NcError> {
        self.toggle(&token, "/favorite", favorite).await
    }

    #[asynced]
    pub async fn set_archived(
        &self,
        token: String,
        archived: bool,
    ) -> Result<Conversation, NcError> {
        self.toggle(&token, "/archive", archived).await
    }

    /// Important conversations notify even when the user status is "Do not disturb".
    #[asynced]
    pub async fn set_important(
        &self,
        token: String,
        important: bool,
    ) -> Result<Conversation, NcError> {
        self.toggle(&token, "/important", important).await
    }

    /// Sensitive conversations hide message previews in the list and notifications.
    #[asynced]
    pub async fn set_sensitive(
        &self,
        token: String,
        sensitive: bool,
    ) -> Result<Conversation, NcError> {
        self.toggle(&token, "/sensitive", sensitive).await
    }

    #[asynced]
    pub async fn set_notification_level(
        &self,
        token: String,
        level: NotificationLevel,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .post(&room_path(&token, "/notify"))
            .json(&serde_json::json!({ "level": level }))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn set_call_notifications(
        &self,
        token: String,
        level: CallNotificationLevel,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .post(&room_path(&token, "/notify-calls"))
            .json(&serde_json::json!({ "level": level }))
            .send()
            .await?
            .data)
    }

    /// Replace the conversation's tags (see `ConversationTagService`).
    #[asynced]
    pub async fn set_tags(
        &self,
        token: String,
        tag_ids: Vec<String>,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .post(&room_path(&token, "/tags"))
            .json(&serde_json::json!({ "tagIds": tag_ids }))
            .send()
            .await?
            .data)
    }

    /// `POST` to enable, `DELETE` to disable.
    async fn toggle(
        &self,
        token: &str,
        suffix: &str,
        enable: bool,
    ) -> Result<Conversation, NcError> {
        let path = room_path(token, suffix);
        let request = if enable {
            self.client.post(&path)
        } else {
            self.client.delete(&path)
        };
        Ok(request.send().await?.data)
    }
}
