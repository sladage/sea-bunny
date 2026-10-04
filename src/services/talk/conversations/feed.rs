//! Keeps the conversation list up to date by polling in the background.
//!
//! Incremental polls only return conversations modified since the previous poll.
//! Removals (left, deleted, kicked) are not reported by the API, so every few
//! polls a full refresh compares the complete list with the known tokens.

use std::{cell::RefCell, collections::HashSet, rc::Rc, time::Duration};

use eventful_rs::*;
use tokio::sync::Notify;

use super::conversation::fetch_conversations;
use crate::{
    dto::talk::{Conversation, ConversationListQuery},
    services::ncclient::{NCClient, NCClientShard, NcError},
};

use_shard!(shard = NCClientShard);

#[events]
pub trait ConversationFeedEvents {
    /// Conversations that changed since the previous poll. After a full refresh
    /// this is the complete list.
    fn on_conversations_changed(&self, conversations: Vec<Conversation>, full_refresh: bool) {}

    /// Conversations the user no longer has access to, detected on a full refresh.
    fn on_conversations_removed(&self, tokens: Vec<String>) {}

    /// The number of pending federation invitations changed.
    fn on_federation_invites_changed(&self, pending: i64) {}

    /// A poll failed. Transient errors are retried on the next interval; the feed
    /// stops after authentication errors.
    fn on_feed_error(&self, error: NcError) {}
}

#[derive(Debug, Clone)]
pub struct ConversationFeedOptions {
    pub interval: Duration,
    /// Do a full refresh every this many polls (the first poll is always full).
    pub full_refresh_every: u32,
    pub include_status: bool,
}

impl Default for ConversationFeedOptions {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            full_refresh_every: 10,
            include_status: true,
        }
    }
}

#[eventful(ConversationFeedEvents)]
pub struct ConversationFeedService {
    client: ShardRc<NCClient>,
    task: RefCell<Option<TaskHandle>>,
    /// Wakes the polling loop for an immediate full refresh.
    refresh: Rc<Notify>,
}

#[asynchronize(pub)]
impl ConversationFeedService {
    pub async fn new(client: &ShardRcHandle<NCClient>) -> Result<ShardRcHandle<Self>, NcError> {
        crate::services::bind_service(client, |client| Self {
            client,
            task: RefCell::new(None),
            refresh: Rc::new(Notify::new()),
            events: Default::default(),
        })
        .await
    }

    /// Start polling, starting with a full refresh. Restarts a running feed.
    #[asynced]
    pub fn start(&self, options: ConversationFeedOptions) -> Result<(), NcError> {
        let refresh = self.refresh.clone();
        let task = ShardRc::spawn_owned(&ShardRc::try_from_ref(self)?, |this| {
            run_feed(this, options, refresh)
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

    /// Poll now with a full refresh instead of waiting for the next interval.
    #[asynced]
    pub fn refresh(&self) {
        self.refresh.notify_one();
    }
}

#[derive(Default)]
struct FeedState {
    known_tokens: HashSet<String>,
    modified_since: i64,
    pending_invites: Option<i64>,
}

impl FeedState {
    /// Update the state with a poll result and emit the resulting events.
    fn apply(
        &mut self,
        feed: &ConversationFeedService,
        list: crate::dto::talk::ConversationList,
        full_refresh: bool,
    ) {
        if let Some(pending) = list.pending_federation_invites
            && self.pending_invites != Some(pending)
        {
            self.pending_invites = Some(pending);
            feed.events.on_federation_invites_changed().emit(pending);
        }

        let tokens = list.conversations.iter().map(|c| c.token.clone());
        if full_refresh {
            let current: HashSet<String> = tokens.collect();
            let removed: Vec<String> = self.known_tokens.difference(&current).cloned().collect();
            self.known_tokens = current;
            if !removed.is_empty() {
                feed.events.on_conversations_removed().emit(removed);
            }
        } else {
            self.known_tokens.extend(tokens);
        }
        if let Some(modified_before) = list.modified_before {
            self.modified_since = modified_before;
        }
        if full_refresh || !list.conversations.is_empty() {
            feed.events
                .on_conversations_changed()
                .emit(list.conversations, full_refresh);
        }
    }
}

async fn run_feed(
    this: ShardWeak<ConversationFeedService>,
    options: ConversationFeedOptions,
    refresh: Rc<Notify>,
) {
    let full_refresh_every = options.full_refresh_every.max(1);
    let mut state = FeedState::default();
    let mut polls = 0u32;
    let mut force_full = true;

    loop {
        let full_refresh = force_full || polls.is_multiple_of(full_refresh_every);
        let query = ConversationListQuery {
            modified_since: if full_refresh {
                0
            } else {
                state.modified_since
            },
            include_status: options.include_status,
            include_last_message: true,
            // Background polling must not keep the user "online".
            no_status_update: true,
        };
        let Some(client) = this.upgrade().map(|feed| feed.client.clone()) else {
            return;
        };
        let result = fetch_conversations(&client, &query).await;
        drop(client);
        let Some(feed) = this.upgrade() else { return };

        match result {
            Ok(list) => {
                state.apply(&feed, list, full_refresh);
                polls = polls.wrapping_add(1);
                force_full = false;
            }
            Err(error) => {
                let fatal = matches!(error, NcError::Unauthorized | NcError::NotAuthenticated);
                feed.events.on_feed_error().emit(error);
                if fatal {
                    // Ending the task marks it finished; `is_running` reports that.
                    return;
                }
                // The next successful poll re-synchronises completely.
                force_full = true;
            }
        }
        drop(feed);

        tokio::select! {
            _ = tokio::time::sleep(options.interval) => {}
            _ = refresh.notified() => force_full = true,
        }
    }
}
