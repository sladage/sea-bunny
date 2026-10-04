//! Listing, creating and removing conversations.

use eventful_rs::*;

use super::{ApiVersion, bind_service, room_path, talk_path};
use crate::{
    dto::talk::{
        Conversation, ConversationList, ConversationListQuery, CreateConversation,
        CreatedConversation,
    },
    services::ncclient::{NCClient, NCClientShard, NcError},
};

use_shard!(shard = NCClientShard);

#[eventful]
pub struct ConversationService {
    client: ShardRc<NCClient>,
}

#[asynchronize(pub)]
impl ConversationService {
    pub async fn new(client: &ShardRcHandle<NCClient>) -> Result<ShardRcHandle<Self>, NcError> {
        bind_service(client, |client| Self {
            client,
            events: Default::default(),
        })
        .await
    }

    /// Conversations of the current user. Use `ConversationList::modified_before` as
    /// the next `modified_since` to only fetch changes.
    #[asynced]
    pub async fn list(&self, query: ConversationListQuery) -> Result<ConversationList, NcError> {
        let response = self
            .client
            .get(&talk_path(ApiVersion::V4, "room"))
            .query(&query)
            .send::<Vec<Conversation>>()
            .await?;
        Ok(ConversationList {
            modified_before: response.header_i64("X-Nextcloud-Talk-Modified-Before"),
            pending_federation_invites: response.header_i64("X-Nextcloud-Talk-Federation-Invites"),
            conversations: response.data,
        })
    }

    #[asynced]
    pub async fn get(&self, token: String) -> Result<Conversation, NcError> {
        Ok(self.client.get(&room_path(&token, "")).send().await?.data)
    }

    /// The user's "Note to self" conversation, created on first access.
    #[asynced]
    pub async fn note_to_self(&self) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .get(&talk_path(ApiVersion::V4, "room/note-to-self"))
            .send()
            .await?
            .data)
    }

    /// Open conversations the user can join, optionally filtered by name.
    #[asynced]
    pub async fn listed(&self, search: Option<String>) -> Result<Vec<Conversation>, NcError> {
        Ok(self
            .client
            .get(&talk_path(ApiVersion::V4, "listed-room"))
            .query(&[("searchTerm", search.unwrap_or_default())])
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn create(
        &self,
        request: CreateConversation,
    ) -> Result<CreatedConversation, NcError> {
        Ok(self
            .client
            .post(&talk_path(ApiVersion::V4, "room"))
            .json(&request)
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn rename(&self, token: String, name: String) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .put(&room_path(&token, ""))
            .json(&serde_json::json!({ "roomName": name }))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn set_description(
        &self,
        token: String,
        description: String,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .put(&room_path(&token, "/description"))
            .json(&serde_json::json!({ "description": description }))
            .send()
            .await?
            .data)
    }

    /// Delete the conversation for everyone (moderators only).
    #[asynced]
    pub async fn delete(&self, token: String) -> Result<(), NcError> {
        self.client
            .delete(&room_path(&token, ""))
            .send_discarding_data()
            .await?;
        Ok(())
    }
}
