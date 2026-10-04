//! Polls notifications in the background, so the client can show desktop
//! notifications for mentions, calls and reminders while a conversation is not open.

use std::{cell::RefCell, collections::HashSet, time::Duration};

use eventful_rs::*;

use super::notification::fetch_notifications;
use crate::{
    dto::notifications::Notification,
    services::ncclient::{NCClient, NCClientShard, NcError},
};

use_shard!(shard = NCClientShard);

#[events]
pub trait NotificationFeedEvents {
    /// The notification list changed; this is the complete list, newest first.
    fn on_notifications_changed(&self, notifications: Vec<Notification>) {}

    /// Notifications that appeared since the previous poll. Notifications already
    /// present when the feed starts are not reported here.
    fn on_new_notifications(&self, notifications: Vec<Notification>) {}

    /// Ids of notifications that disappeared (dismissed here or on another device).
    fn on_notifications_removed(&self, ids: Vec<i64>) {}

    /// A poll failed. Transient errors are retried on the next interval; the feed
    /// stops after authentication errors.
    fn on_feed_error(&self, error: NcError) {}
}

#[derive(Debug, Clone)]
pub struct NotificationFeedOptions {
    /// Nextcloud's own clients use the `core => pollinterval` capability (30 s default).
    pub interval: Duration,
}

impl Default for NotificationFeedOptions {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
        }
    }
}

#[eventful(NotificationFeedEvents)]
pub struct NotificationFeedService {
    client: ShardRc<NCClient>,
    task: RefCell<Option<TaskHandle>>,
}

#[asynchronize(pub)]
impl NotificationFeedService {
    pub async fn new(client: &ShardRcHandle<NCClient>) -> Result<ShardRcHandle<Self>, NcError> {
        crate::services::bind_service(client, |client| Self {
            client,
            task: RefCell::new(None),
            events: Default::default(),
        })
        .await
    }

    /// Start polling. Restarts a running feed.
    #[asynced]
    pub fn start(&self, options: NotificationFeedOptions) -> Result<(), NcError> {
        let task = ShardRc::spawn_owned(&ShardRc::try_from_ref(self)?, |this| {
            run_feed(this, options)
        })?;
        if let Some(previous) = self.task.replace(Some(task)) {
            previous.abort();
        }
        Ok(())
    }

    #[asynced]
    pub fn stop(&self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }

    #[asynced]
    pub fn is_running(&self) -> bool {
        self.task
            .borrow()
            .as_ref()
            .is_some_and(|task| !task.is_finished())
    }
}

#[derive(Default)]
struct FeedState {
    etag: Option<String>,
    /// `None` until the first successful poll, which only sets the baseline.
    known_ids: Option<HashSet<i64>>,
}

impl FeedState {
    fn apply(&mut self, feed: &NotificationFeedService, notifications: Vec<Notification>) {
        let current: HashSet<i64> = notifications.iter().map(|n| n.notification_id).collect();
        if let Some(known) = &self.known_ids {
            let new: Vec<Notification> = notifications
                .iter()
                .filter(|n| !known.contains(&n.notification_id))
                .cloned()
                .collect();
            let removed: Vec<i64> = known.difference(&current).copied().collect();
            if !removed.is_empty() {
                feed.events.on_notifications_removed().emit(removed);
            }
            if !new.is_empty() {
                feed.events.on_new_notifications().emit(new);
            }
        }
        self.known_ids = Some(current);
        feed.events.on_notifications_changed().emit(notifications);
    }
}

async fn run_feed(this: ShardWeak<NotificationFeedService>, options: NotificationFeedOptions) {
    let mut state = FeedState::default();
    loop {
        let Some(client) = this.upgrade().map(|feed| feed.client.clone()) else {
            return;
        };
        let result = fetch_notifications(&client, state.etag.as_deref()).await;
        drop(client);
        let Some(feed) = this.upgrade() else { return };

        match result {
            // Unchanged since the previous poll.
            Ok(None) => {}
            Ok(Some(list)) => {
                state.etag = list.etag;
                state.apply(&feed, list.notifications);
            }
            Err(error) => {
                let fatal = matches!(error, NcError::Unauthorized | NcError::NotAuthenticated);
                feed.events.on_feed_error().emit(error);
                if fatal {
                    return;
                }
            }
        }
        drop(feed);
        tokio::time::sleep(options.interval).await;
    }
}
