//! The current user's conversation tags (groups in the conversation list).
//! Assign tags with `ConversationPreferenceService::set_tags`.

use eventful_rs::*;

use crate::{
    dto::talk::ConversationTag,
    services::{
        ncclient::NcError,
        talk::{ApiVersion, talk_path},
    },
};

service! {
    pub struct ConversationTagService;
}

#[asynchronize(pub)]
impl ConversationTagService {
    #[asynced]
    pub async fn list(&self) -> Result<Vec<ConversationTag>, NcError> {
        Ok(self.client.get(&tags_path("")).send().await?.data)
    }

    #[asynced]
    pub async fn create(&self, name: String) -> Result<ConversationTag, NcError> {
        Ok(self
            .client
            .post(&tags_path(""))
            .json(&serde_json::json!({ "name": name }))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn rename(&self, tag_id: String, name: String) -> Result<ConversationTag, NcError> {
        Ok(self
            .client
            .put(&tags_path(&format!("/{tag_id}")))
            .json(&serde_json::json!({ "name": name }))
            .send()
            .await?
            .data)
    }

    #[asynced]
    pub async fn delete(&self, tag_id: String) -> Result<(), NcError> {
        self.client
            .delete(&tags_path(&format!("/{tag_id}")))
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Set the order of all tags. Returns them in the new order.
    #[asynced]
    pub async fn reorder(&self, ordered_ids: Vec<String>) -> Result<Vec<ConversationTag>, NcError> {
        Ok(self
            .client
            .put(&tags_path("/reorder"))
            .json(&serde_json::json!({ "orderedIds": ordered_ids }))
            .send()
            .await?
            .data)
    }

    /// Remember whether the tag's group is collapsed in the conversation list.
    #[asynced]
    pub async fn set_collapsed(
        &self,
        tag_id: String,
        collapsed: bool,
    ) -> Result<ConversationTag, NcError> {
        Ok(self
            .client
            .put(&tags_path(&format!("/{tag_id}/collapsed")))
            .json(&serde_json::json!({ "collapsed": collapsed }))
            .send()
            .await?
            .data)
    }
}

fn tags_path(suffix: &str) -> String {
    talk_path(ApiVersion::V4, format!("tags{suffix}"))
}
