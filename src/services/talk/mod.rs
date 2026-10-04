//! Nextcloud Talk API services.
//!
//! Each service covers one API area and lives on [`NCClientShard`] next to the
//! [`NCClient`] it uses, so it can call the client's shard-local request API
//! directly. Other shards use the services through their handles.

pub mod chat;
pub mod chat_feed;
pub mod conversation_sessions;
pub mod conversations;
pub mod participants;

#[cfg(test)]
mod tests;

use eventful_rs::*;

use crate::services::ncclient::{NCClient, NCClientShard, NcError};

pub use chat::ChatService;
pub use chat_feed::ChatFeedService;
pub use conversation_sessions::ConversationSessionService;
pub use conversations::ConversationService;
pub use participants::ParticipantService;

/// Talk versions its endpoints individually.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ApiVersion {
    V1,
    V4,
}

/// Path of a Talk OCS endpoint, e.g. `talk_path(ApiVersion::V4, "room")`.
pub(crate) fn talk_path(version: ApiVersion, path: impl std::fmt::Display) -> String {
    let version = match version {
        ApiVersion::V1 => "v1",
        ApiVersion::V4 => "v4",
    };
    format!("ocs/v2.php/apps/spreed/api/{version}/{path}")
}

/// Path of a conversation endpoint: `room/{token}{suffix}`, where `suffix` is
/// empty or starts with `/`.
pub(crate) fn room_path(token: &str, suffix: &str) -> String {
    talk_path(ApiVersion::V4, format!("room/{token}{suffix}"))
}

/// Path of a chat endpoint: `chat/{token}{suffix}`, where `suffix` is empty or
/// starts with `/`.
pub(crate) fn chat_path(token: &str, suffix: &str) -> String {
    talk_path(ApiVersion::V1, format!("chat/{token}{suffix}"))
}

/// Routes events to listeners of one conversation, by token.
///
/// Connect with `.labelled(ConversationLabel::new(token))` to only receive events
/// for that conversation; unlabelled connections receive all of them.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConversationLabel(pub String);

impl ConversationLabel {
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }
}

impl EventLabel for ConversationLabel {
    fn matches(&self, emitted: &Self) -> bool {
        self == emitted
    }
}

/// Bind a service on the client's shard, handing it a local reference to the client.
pub(crate) async fn bind_service<S, F>(
    client: &ShardRcHandle<NCClient>,
    make: F,
) -> Result<ShardRcHandle<S>, NcError>
where
    S: Eventful<Shard = NCClientShard> + HasEvents<S::EventSetType> + 'static,
    F: FnOnce(ShardRc<NCClient>) -> S + Send + 'static,
{
    client
        .deferred_upgrade_in_shard(async move |client: &NCClient| {
            let client = ShardRc::try_from_ref(client)?;
            Ok(ShardRc::try_bind(make(client))?.to_handle())
        })
        .await
}

/// Handles to all Talk services.
#[derive(Clone)]
pub struct TalkServices {
    pub conversations: ShardRcHandle<ConversationService>,
    pub sessions: ShardRcHandle<ConversationSessionService>,
    pub participants: ShardRcHandle<ParticipantService>,
    pub chat: ShardRcHandle<ChatService>,
    pub chat_feed: ShardRcHandle<ChatFeedService>,
}

impl TalkServices {
    pub async fn new(client: &ShardRcHandle<NCClient>) -> Result<Self, NcError> {
        Ok(Self {
            conversations: ConversationService::new(client).await?,
            sessions: ConversationSessionService::new(client).await?,
            participants: ParticipantService::new(client).await?,
            chat: ChatService::new(client).await?,
            chat_feed: ChatFeedService::new(client).await?,
        })
    }
}
