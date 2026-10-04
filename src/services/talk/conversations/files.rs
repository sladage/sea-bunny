//! Conversations attached to files and file shares.

use eventful_rs::*;
use serde::Deserialize;

use crate::{
    dto::talk::{ShareAuthConversation, ShareConversation},
    services::{
        ncclient::NcError,
        talk::{ApiVersion, talk_path},
    },
};

service! {
    pub struct FileConversationService;
}

#[asynchronize(pub)]
impl FileConversationService {
    /// Token of the conversation about a file (created on first access).
    #[asynced]
    pub async fn for_file(&self, file_id: String) -> Result<String, NcError> {
        #[derive(Deserialize)]
        struct Token {
            token: String,
        }

        Ok(self
            .client
            .get(&talk_path(ApiVersion::V1, format!("file/{file_id}")))
            .send::<Token>()
            .await?
            .data
            .token)
    }

    #[asynced]
    pub async fn for_public_share(
        &self,
        share_token: String,
    ) -> Result<ShareConversation, NcError> {
        Ok(self
            .client
            .get(&talk_path(
                ApiVersion::V1,
                format!("publicshare/{share_token}"),
            ))
            .send()
            .await?
            .data)
    }

    /// Start a video verification for a share protected with "password by Talk".
    #[asynced]
    pub async fn request_share_password(
        &self,
        share_token: String,
    ) -> Result<ShareAuthConversation, NcError> {
        Ok(self
            .client
            .post(&talk_path(ApiVersion::V1, "publicshareauth"))
            .json(&serde_json::json!({ "shareToken": share_token }))
            .send()
            .await?
            .data)
    }
}
