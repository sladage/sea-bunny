//! Attendees of a conversation: listing, inviting, removing and their permissions.

use eventful_rs::*;
use serde::Deserialize;

use crate::{
    dto::talk::{
        Conversation, ConversationType, EmailImportResult, Participant, ParticipantSource,
        PermissionMethod, Permissions,
    },
    services::{ncclient::NcError, talk::room_path},
};

service! {
    pub struct ParticipantService;
}

#[asynchronize(pub)]
impl ParticipantService {
    #[asynced]
    pub async fn list(
        &self,
        token: String,
        include_status: bool,
    ) -> Result<Vec<Participant>, NcError> {
        Ok(self
            .client
            .get(&room_path(&token, "/participants"))
            .query(&[("includeStatus", include_status)])
            .send()
            .await?
            .data)
    }

    /// Invite an actor. Returns the conversation type afterwards: adding someone
    /// to a one-to-one conversation creates a new group conversation.
    #[asynced]
    pub async fn add(
        &self,
        token: String,
        participant: String,
        source: ParticipantSource,
    ) -> Result<ConversationType, NcError> {
        #[derive(Deserialize)]
        struct Added {
            #[serde(rename = "type")]
            conversation_type: ConversationType,
        }

        let added = self
            .client
            .post(&room_path(&token, "/participants"))
            .json(&serde_json::json!({ "newParticipant": participant, "source": source }))
            .send::<Added>()
            .await?
            .data;
        Ok(added.conversation_type)
    }

    /// Remove another attendee (moderators only).
    #[asynced]
    pub async fn remove(&self, token: String, attendee_id: i64) -> Result<(), NcError> {
        self.client
            .delete(&room_path(&token, "/attendees"))
            .query(&[("attendeeId", attendee_id)])
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Leave the conversation as the current user.
    #[asynced]
    pub async fn leave(&self, token: String) -> Result<(), NcError> {
        self.client
            .delete(&room_path(&token, "/participants/self"))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    #[asynced]
    pub async fn promote_moderator(&self, token: String, attendee_id: i64) -> Result<(), NcError> {
        self.client
            .post(&room_path(&token, "/moderators"))
            .json(&serde_json::json!({ "attendeeId": attendee_id }))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    #[asynced]
    pub async fn demote_moderator(&self, token: String, attendee_id: i64) -> Result<(), NcError> {
        self.client
            .delete(&room_path(&token, "/moderators"))
            .query(&[("attendeeId", attendee_id)])
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Change one attendee's permissions. Returns the updated participant list.
    #[asynced]
    pub async fn set_permissions(
        &self,
        token: String,
        attendee_id: i64,
        method: PermissionMethod,
        permissions: Permissions,
    ) -> Result<Vec<Participant>, NcError> {
        Ok(self
            .client
            .put(&room_path(&token, "/attendees/permissions"))
            .json(&serde_json::json!({
                "attendeeId": attendee_id,
                "method": method,
                "permissions": permissions,
            }))
            .send()
            .await?
            .data)
    }

    /// Change the permissions of all attendees at once.
    #[asynced]
    pub async fn set_all_permissions(
        &self,
        token: String,
        method: PermissionMethod,
        permissions: Permissions,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .put(&room_path(&token, "/attendees/permissions/all"))
            .json(&serde_json::json!({ "method": method, "permissions": permissions }))
            .send()
            .await?
            .data)
    }

    /// Resend email invitations, to one attendee or (with `None`) to all pending ones.
    #[asynced]
    pub async fn resend_invitations(
        &self,
        token: String,
        attendee_id: Option<i64>,
    ) -> Result<(), NcError> {
        self.client
            .post(&room_path(&token, "/participants/resend-invitations"))
            .json(&serde_json::json!({ "attendeeId": attendee_id }))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Invite email guests from a CSV file with an `email` and optional `name`
    /// column. With `test_run`, only validates the file.
    #[asynced]
    pub async fn import_emails(
        &self,
        token: String,
        csv: Vec<u8>,
        test_run: bool,
    ) -> Result<EmailImportResult, NcError> {
        let file = reqwest::multipart::Part::bytes(csv)
            .file_name("participants.csv")
            .mime_str("text/csv")
            .map_err(NcError::from)?;
        let form = reqwest::multipart::Form::new()
            .part("file", file)
            .text("testRun", if test_run { "1" } else { "0" });
        Ok(self
            .client
            .post(&room_path(&token, "/import-emails"))
            .multipart(form)
            .send()
            .await?
            .data)
    }
}
