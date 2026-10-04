//! Conversation avatars. Changing them requires moderator permissions and is not
//! possible for one-to-one conversations.

use eventful_rs::*;
use reqwest::multipart::{Form, Part};

use crate::{
    dto::talk::Conversation,
    services::{
        ncclient::{Download, NcError},
        talk::{ApiVersion, talk_path},
    },
};

service! {
    pub struct ConversationAvatarService;
}

#[asynchronize(pub)]
impl ConversationAvatarService {
    /// The avatar image (custom, or generated from the conversation). Re-fetch when
    /// `Conversation::avatar_version` changes.
    #[asynced]
    pub async fn get(&self, token: String, dark_theme: bool) -> Result<Download, NcError> {
        let suffix = if dark_theme { "/dark" } else { "" };
        self.client
            .get(&avatar_path(&token, suffix))
            .send_download()
            .await
    }

    /// Upload a square PNG or JPEG image.
    #[asynced]
    pub async fn upload(
        &self,
        token: String,
        image: Vec<u8>,
        mime_type: String,
    ) -> Result<Conversation, NcError> {
        let part = Part::bytes(image)
            .file_name("avatar")
            .mime_str(&mime_type)
            .map_err(NcError::from)?;
        Ok(self
            .client
            .post(&avatar_path(&token, ""))
            .multipart(Form::new().part("file", part))
            .send()
            .await?
            .data)
    }

    /// Use an emoji on a colored background. `color` is a hex code without `#`;
    /// `None` picks a default.
    #[asynced]
    pub async fn set_emoji(
        &self,
        token: String,
        emoji: String,
        color: Option<String>,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .post(&avatar_path(&token, "/emoji"))
            .json(&serde_json::json!({ "emoji": emoji, "color": color }))
            .send()
            .await?
            .data)
    }

    /// Remove the custom avatar, going back to the generated one.
    #[asynced]
    pub async fn delete(&self, token: String) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .delete(&avatar_path(&token, ""))
            .send()
            .await?
            .data)
    }
}

fn avatar_path(token: &str, suffix: &str) -> String {
    talk_path(ApiVersion::V1, format!("room/{token}/avatar{suffix}"))
}
