//! Conversation services: the conversations themselves, their settings,
//! participants and everything attached to them.

pub mod access;
pub mod avatars;
pub mod bans;
pub mod breakout_rooms;
pub mod conversation;
pub mod feed;
pub mod files;
pub mod meetings;
pub mod participants;
pub mod preferences;
pub mod sessions;
pub mod settings;
pub mod tags;

use eventful_rs::ShardRcHandle;

pub use access::ConversationAccessService;
pub use avatars::ConversationAvatarService;
pub use bans::BanService;
pub use breakout_rooms::BreakoutRoomService;
pub use conversation::ConversationService;
pub use feed::{ConversationFeedOptions, ConversationFeedService};
pub use files::FileConversationService;
pub use meetings::MeetingService;
pub use participants::ParticipantService;
pub use preferences::ConversationPreferenceService;
pub use sessions::ConversationSessionService;
pub use settings::ConversationSettingsService;
pub use tags::ConversationTagService;

use crate::services::ncclient::{NCClient, NcError};

#[derive(Clone)]
pub struct ConversationServices {
    pub conversations: ShardRcHandle<ConversationService>,
    pub feed: ShardRcHandle<ConversationFeedService>,
    pub access: ShardRcHandle<ConversationAccessService>,
    pub settings: ShardRcHandle<ConversationSettingsService>,
    pub preferences: ShardRcHandle<ConversationPreferenceService>,
    pub tags: ShardRcHandle<ConversationTagService>,
    pub avatars: ShardRcHandle<ConversationAvatarService>,
    pub sessions: ShardRcHandle<ConversationSessionService>,
    pub participants: ShardRcHandle<ParticipantService>,
    pub bans: ShardRcHandle<BanService>,
    pub breakout_rooms: ShardRcHandle<BreakoutRoomService>,
    pub meetings: ShardRcHandle<MeetingService>,
    pub files: ShardRcHandle<FileConversationService>,
}

impl ConversationServices {
    pub async fn new(client: &ShardRcHandle<NCClient>) -> Result<Self, NcError> {
        Ok(Self {
            conversations: ConversationService::new(client).await?,
            feed: ConversationFeedService::new(client).await?,
            access: ConversationAccessService::new(client).await?,
            settings: ConversationSettingsService::new(client).await?,
            preferences: ConversationPreferenceService::new(client).await?,
            tags: ConversationTagService::new(client).await?,
            avatars: ConversationAvatarService::new(client).await?,
            sessions: ConversationSessionService::new(client).await?,
            participants: ParticipantService::new(client).await?,
            bans: BanService::new(client).await?,
            breakout_rooms: BreakoutRoomService::new(client).await?,
            meetings: MeetingService::new(client).await?,
            files: FileConversationService::new(client).await?,
        })
    }
}
