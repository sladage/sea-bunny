//! One-shot chat operations: loading history, sending, editing and deleting
//! messages, and read markers. Live updates come from [`super::ChatFeedService`].

use eventful_rs::*;

use crate::{
    dto::talk::{
        ChatMessage, ChatPage, ChatSummary, Conversation, HistoryQuery, MentionSuggestion,
        ReceiveMessagesQuery, SendMessage,
    },
    services::{ncclient::NcError, talk::chat_path},
};

service! {
    pub struct ChatService;
}

#[asynchronize(pub)]
impl ChatService {
    /// Load a page of messages without waiting for new ones.
    #[asynced]
    pub async fn history(&self, token: String, query: HistoryQuery) -> Result<ChatPage, NcError> {
        let response = self
            .client
            .get(&chat_path(&token, ""))
            .query(&ReceiveMessagesQuery::from(&query))
            .send_optional::<Vec<ChatMessage>>()
            .await?;
        Ok(ChatPage {
            last_given: response.header_i64("X-Chat-Last-Given"),
            last_common_read: response.header_i64("X-Chat-Last-Common-Read"),
            // 304: nothing beyond `last_known_message_id`.
            messages: response.data.unwrap_or_default(),
        })
    }

    /// Messages around `message_id` (for jumping to a search result or reply).
    #[asynced]
    pub async fn context(
        &self,
        token: String,
        message_id: i64,
        limit: u32,
        thread_id: Option<i64>,
    ) -> Result<Vec<ChatMessage>, NcError> {
        let response = self
            .client
            .get(&chat_path(&token, &format!("/{message_id}/context")))
            .query(&[("limit", limit)])
            .query(&[("threadId", thread_id.unwrap_or(0))])
            .send_optional::<Vec<ChatMessage>>()
            .await?;
        Ok(response.data.unwrap_or_default())
    }

    /// Send a message. Returns the stored message, or `None` if the server did not
    /// echo it (it will still arrive through the chat feed).
    #[asynced]
    pub async fn send(
        &self,
        token: String,
        message: SendMessage,
    ) -> Result<Option<ChatMessage>, NcError> {
        Ok(self
            .client
            .post(&chat_path(&token, ""))
            .json(&message)
            .send()
            .await?
            .data)
    }

    /// Edit a message. Returns the system message announcing the edit; its `parent`
    /// is the edited message.
    #[asynced]
    pub async fn edit(
        &self,
        token: String,
        message_id: i64,
        message: String,
    ) -> Result<ChatMessage, NcError> {
        Ok(self
            .client
            .put(&chat_path(&token, &format!("/{message_id}")))
            .json(&serde_json::json!({ "message": message }))
            .send()
            .await?
            .data)
    }

    /// Delete a message. Returns the system message announcing the deletion; its
    /// `parent` is the deleted message.
    #[asynced]
    pub async fn delete(&self, token: String, message_id: i64) -> Result<ChatMessage, NcError> {
        Ok(self
            .client
            .delete(&chat_path(&token, &format!("/{message_id}")))
            .send()
            .await?
            .data)
    }

    /// Delete all messages (moderators only). Returns the resulting system message.
    #[asynced]
    pub async fn clear_history(&self, token: String) -> Result<ChatMessage, NcError> {
        Ok(self
            .client
            .delete(&chat_path(&token, ""))
            .send()
            .await?
            .data)
    }

    /// Set the read marker to `last_read_message`, or to the newest message with `None`.
    #[asynced]
    pub async fn set_read_marker(
        &self,
        token: String,
        last_read_message: Option<i64>,
    ) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .post(&chat_path(&token, "/read"))
            .json(&serde_json::json!({ "lastReadMessage": last_read_message }))
            .send()
            .await?
            .data)
    }

    /// Mark the conversation unread, moving the read marker before the last message.
    #[asynced]
    pub async fn mark_unread(&self, token: String) -> Result<Conversation, NcError> {
        Ok(self
            .client
            .delete(&chat_path(&token, "/read"))
            .send()
            .await?
            .data)
    }

    /// Suggestions for completing an `@mention` while typing.
    #[asynced]
    pub async fn mention_suggestions(
        &self,
        token: String,
        search: String,
        limit: u32,
        include_status: bool,
    ) -> Result<Vec<MentionSuggestion>, NcError> {
        Ok(self
            .client
            .get(&chat_path(&token, "/mentions"))
            .query(&[("search", search)])
            .query(&[("limit", limit)])
            .query(&[("includeStatus", include_status)])
            .send()
            .await?
            .data)
    }

    /// Start an AI summary of the messages after `from_message_id`. Returns `None`
    /// when there is too little to summarize. The summary arrives as a
    /// notification once the task finishes.
    #[asynced]
    pub async fn summarize(
        &self,
        token: String,
        from_message_id: i64,
    ) -> Result<Option<ChatSummary>, NcError> {
        Ok(self
            .client
            .post(&chat_path(&token, "/summarize"))
            .json(&serde_json::json!({ "fromMessageId": from_message_id }))
            .send_optional()
            .await?
            .data)
    }
}
