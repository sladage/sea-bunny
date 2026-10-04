//! Conversation-wide policies: default permissions, mentions, message expiration,
//! recording consent, preservation and object binding. Moderators only.

use eventful_rs::*;

use crate::{
    dto::talk::{Conversation, MentionPermissions, Permissions, RecordingConsent},
    services::{ncclient::NcError, talk::room_path},
};

service! {
    pub struct ConversationSettingsService;
}

#[asynchronize(pub)]
impl ConversationSettingsService {
    /// Permissions of attendees without custom permissions. Must include
    /// [`Permissions::CUSTOM`] to restrict below the server default.
    #[asynced]
    pub async fn set_default_permissions(
        &self,
        token: String,
        permissions: Permissions,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .put(&room_path(&token, "/permissions/default"))
            .json(&serde_json::json!({ "permissions": permissions }))
            .send()
            .await?
            .data)
    }

    /// Who may mention `@all`.
    #[asynced]
    pub async fn set_mention_permissions(
        &self,
        token: String,
        permissions: MentionPermissions,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .put(&room_path(&token, "/mention-permissions"))
            .json(&serde_json::json!({ "mentionPermissions": permissions }))
            .send()
            .await?
            .data)
    }

    /// Delete messages `seconds` after posting; 0 disables expiration.
    #[asynced]
    pub async fn set_message_expiration(
        &self,
        token: String,
        seconds: i64,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .post(&room_path(&token, "/message-expiration"))
            .json(&serde_json::json!({ "seconds": seconds }))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn set_recording_consent(
        &self,
        token: String,
        consent: RecordingConsent,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .put(&room_path(&token, "/recording-consent"))
            .json(&serde_json::json!({ "recordingConsent": consent }))
            .send()
            .await?
            .data)
    }

    /// Protect the conversation from deletion, clearing and guest/listable changes
    /// (owners only).
    #[asynced]
    pub async fn set_preserved(
        &self,
        token: String,
        preserved: bool,
    ) -> Result<Conversation, NcError> {
        let path = room_path(&token, "/preserve");
        let request = if preserved {
            self.client.post(&path)
        } else {
            self.client.delete(&path)
        };
        Ok(request.send().await?.data)
    }

    /// Detach the conversation from the object it was created for (e.g. an event),
    /// so it is no longer cleaned up with it.
    #[asynced]
    pub async fn unbind_object(&self, token: String) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .delete(&room_path(&token, "/object"))
            .send()
            .await?
            .data)
    }
}
