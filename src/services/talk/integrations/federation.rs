//! Invitations to conversations on other Nextcloud servers. The number of
//! pending invitations is reported by `ConversationFeedService`.

use eventful_rs::*;

use crate::{
    dto::talk::{Conversation, FederationInvite},
    services::{
        ncclient::NcError,
        talk::{ApiVersion, talk_path},
    },
};

service! {
    pub struct FederationService;
}

#[asynchronize(pub)]
impl FederationService {
    #[asynced]
    pub async fn invitations(&self) -> Result<Vec<FederationInvite>, NcError> {
        Ok(self.client.get(&invitation_path("")).send().await?.data)
    }

    /// Accept an invitation. Returns the local proxy conversation.
    #[asynced]
    pub async fn accept(&self, invitation_id: i64) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .post(&invitation_path(&format!("/{invitation_id}")))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn reject(&self, invitation_id: i64) -> Result<(), NcError> {
        self.client
            .delete(&invitation_path(&format!("/{invitation_id}")))
            .send_discarding_data()
            .await?;
        Ok(())
    }
}

fn invitation_path(suffix: &str) -> String {
    talk_path(ApiVersion::V1, format!("federation/invitation{suffix}"))
}
