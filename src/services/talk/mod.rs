//! Nextcloud Talk (spreed) API services, grouped by area.
//!
//! Every service is a small `#[eventful]` type on the client's shard; other
//! shards use them through the handles collected in [`TalkServices`].

pub mod calls;
pub mod chat;
pub mod conversations;
pub mod guests;
pub mod integrations;
pub mod user_settings;

#[cfg(test)]
mod tests;

use eventful_rs::*;

use crate::services::ncclient::{NCClient, NcError};

pub use calls::CallServices;
pub use chat::ChatServices;
pub use conversations::ConversationServices;
pub use guests::GuestService;
pub use integrations::IntegrationServices;
pub use user_settings::UserSettingsService;

/// Talk versions its endpoints individually.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ApiVersion {
    V1,
    V3,
    V4,
}

/// Path of a Talk OCS endpoint, e.g. `talk_path(ApiVersion::V4, "room")`.
pub(crate) fn talk_path(version: ApiVersion, path: impl std::fmt::Display) -> String {
    let version = match version {
        ApiVersion::V1 => "v1",
        ApiVersion::V3 => "v3",
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

/// Handles to all Talk services.
#[derive(Clone)]
pub struct TalkServices {
    pub conversations: ConversationServices,
    pub chat: ChatServices,
    pub calls: CallServices,
    pub integrations: IntegrationServices,
    pub user_settings: ShardRcHandle<UserSettingsService>,
    pub guests: ShardRcHandle<GuestService>,
}

impl TalkServices {
    pub async fn new(client: &ShardRcHandle<NCClient>) -> Result<Self, NcError> {
        Ok(Self {
            conversations: ConversationServices::new(client).await?,
            chat: ChatServices::new(client).await?,
            calls: CallServices::new(client).await?,
            integrations: IntegrationServices::new(client).await?,
            user_settings: UserSettingsService::new(client).await?,
            guests: GuestService::new(client).await?,
        })
    }
}
