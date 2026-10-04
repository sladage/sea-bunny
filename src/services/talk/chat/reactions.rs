//! Emoji reactions to chat messages.

use eventful_rs::*;

use crate::{
    dto::{serde_ext::PhpMap, talk::Reactions},
    services::{
        ncclient::NcError,
        talk::{ApiVersion, talk_path},
    },
};

service! {
    pub struct ReactionService;
}

#[asynchronize(pub)]
impl ReactionService {
    /// Who reacted to a message, for one emoji or (with `None`) all of them.
    #[asynced]
    pub async fn list(
        &self,
        token: String,
        message_id: i64,
        reaction: Option<String>,
    ) -> Result<Reactions, NcError> {
        Ok(self
            .client
            .get(&reaction_path(&token, message_id))
            .query(&[("reaction", reaction)])
            .send::<PhpMap<_>>()
            .await?
            .data
            .0)
    }

    /// React with an emoji. Returns the message's reactions afterwards.
    #[asynced]
    pub async fn add(
        &self,
        token: String,
        message_id: i64,
        reaction: String,
    ) -> Result<Reactions, NcError> {
        Ok(self
            .client
            .post(&reaction_path(&token, message_id))
            .json(&serde_json::json!({ "reaction": reaction }))
            .send::<PhpMap<_>>()
            .await?
            .data
            .0)
    }

    /// Remove the current user's reaction. Returns the message's reactions afterwards.
    #[asynced]
    pub async fn remove(
        &self,
        token: String,
        message_id: i64,
        reaction: String,
    ) -> Result<Reactions, NcError> {
        Ok(self
            .client
            .delete(&reaction_path(&token, message_id))
            .query(&[("reaction", reaction)])
            .send::<PhpMap<_>>()
            .await?
            .data
            .0)
    }
}

fn reaction_path(token: &str, message_id: i64) -> String {
    talk_path(ApiVersion::V1, format!("reaction/{token}/{message_id}"))
}
