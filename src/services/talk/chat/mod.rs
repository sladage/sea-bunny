//! Chat services: messages, live updates and everything attached to messages.

pub mod feed;
pub mod messages;
pub mod pins;
pub mod polls;
pub mod reactions;
pub mod reminders;
pub mod scheduled;
pub mod sharing;
pub mod threads;

use eventful_rs::ShardRcHandle;

pub use feed::{ChatFeedOptions, ChatFeedService};
pub use messages::ChatService;
pub use pins::PinService;
pub use polls::PollService;
pub use reactions::ReactionService;
pub use reminders::ReminderService;
pub use scheduled::ScheduledMessageService;
pub use sharing::{ChatShareService, UploadAttachment, UploadedAttachment};
pub use threads::ThreadService;

use crate::services::ncclient::{NCClient, NcError};

#[derive(Clone)]
pub struct ChatServices {
    pub messages: ShardRcHandle<ChatService>,
    pub feed: ShardRcHandle<ChatFeedService>,
    pub sharing: ShardRcHandle<ChatShareService>,
    pub reactions: ShardRcHandle<ReactionService>,
    pub polls: ShardRcHandle<PollService>,
    pub reminders: ShardRcHandle<ReminderService>,
    pub pins: ShardRcHandle<PinService>,
    pub scheduled: ShardRcHandle<ScheduledMessageService>,
    pub threads: ShardRcHandle<ThreadService>,
}

impl ChatServices {
    pub async fn new(client: &ShardRcHandle<NCClient>) -> Result<Self, NcError> {
        Ok(Self {
            messages: ChatService::new(client).await?,
            feed: ChatFeedService::new(client).await?,
            sharing: ChatShareService::new(client).await?,
            reactions: ReactionService::new(client).await?,
            polls: PollService::new(client).await?,
            reminders: ReminderService::new(client).await?,
            pins: PinService::new(client).await?,
            scheduled: ScheduledMessageService::new(client).await?,
            threads: ThreadService::new(client).await?,
        })
    }
}
