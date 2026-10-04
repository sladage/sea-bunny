//! Nextcloud Notifications app services. Not part of Talk, but Talk delivers
//! mentions, incoming calls and reminders through it.

pub mod feed;
pub mod notification;

use eventful_rs::ShardRcHandle;

pub use feed::{NotificationFeedOptions, NotificationFeedService};
pub use notification::NotificationService;

use crate::services::ncclient::{NCClient, NcError};

#[derive(Clone)]
pub struct NotificationServices {
    pub notifications: ShardRcHandle<NotificationService>,
    pub feed: ShardRcHandle<NotificationFeedService>,
}

impl NotificationServices {
    pub async fn new(client: &ShardRcHandle<NCClient>) -> Result<Self, NcError> {
        Ok(Self {
            notifications: NotificationService::new(client).await?,
            feed: NotificationFeedService::new(client).await?,
        })
    }
}

#[cfg(test)]
mod tests;
