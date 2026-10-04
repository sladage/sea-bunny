//! Live chat updates through long polling, one background task per subscribed
//! conversation.
//!
//! Events are labelled with the conversation token: a chat view connects with
//! `.labelled(ConversationLabel::new(token))`, while e.g. an unread counter can
//! connect unlabelled and receive updates for every subscribed conversation.

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    time::Duration,
};

use eventful_rs::*;

use crate::{
    dto::talk::{ChatMessage, ReceiveMessagesQuery},
    services::{
        bind_service,
        ncclient::{NCClient, NCClientShard, NcError},
        talk::{ConversationLabel, chat_path},
    },
};

use_shard!(shard = NCClientShard);

/// Seconds the server holds a request open waiting for new messages (max 60).
const LONG_POLL_SECONDS: u32 = 30;
/// Client-side timeout; must exceed the long-poll time.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(LONG_POLL_SECONDS as u64 + 15);
const RETRY_MIN: Duration = Duration::from_secs(1);
const RETRY_MAX: Duration = Duration::from_secs(30);
const PAGE_SIZE: u32 = 100;

#[events]
pub trait ChatFeedEvents {
    /// New messages, oldest first. Includes system messages for edits, deletions,
    /// reactions, etc.; their `parent` carries the affected message.
    #[with_label(ConversationLabel)]
    fn on_messages(&self, token: String, messages: Vec<ChatMessage>) {}

    /// The newest message read by every participant changed.
    #[with_label(ConversationLabel)]
    fn on_read_marker(&self, token: String, last_common_read: i64) {}

    /// The feed stopped because of a non-recoverable error (no access anymore,
    /// conversation deleted, session expired, ...). Subscribe again to restart it.
    #[with_label(ConversationLabel)]
    fn on_feed_stopped(&self, token: String, error: NcError) {}
}

/// Where and how to start a feed.
#[derive(Debug, Clone, Default)]
pub struct ChatFeedOptions {
    /// Only messages newer than this are delivered. Usually the newest message
    /// loaded with `ChatService::history`.
    pub last_known_message_id: i64,
    /// Move the read marker along with received messages (while the chat is visible).
    pub set_read_marker: bool,
    /// Don't mark the user as online (while the application is in the background).
    pub no_status_update: bool,
    pub thread_id: Option<i64>,
}

#[eventful(ChatFeedEvents)]
pub struct ChatFeedService {
    client: ShardRc<NCClient>,
    feeds: RefCell<HashMap<String, Feed>>,
    next_feed_id: Cell<u64>,
}

struct Feed {
    /// Distinguishes a replaced feed from its successor for the same token.
    id: u64,
    /// Owned by the service: aborted when it is destroyed.
    task: TaskHandle,
}

#[asynchronize(pub)]
impl ChatFeedService {
    pub async fn new(client: &ShardRcHandle<NCClient>) -> Result<ShardRcHandle<Self>, NcError> {
        bind_service(client, |client| Self {
            client,
            feeds: RefCell::new(HashMap::new()),
            next_feed_id: Cell::new(0),
            events: Default::default(),
        })
        .await
    }

    /// Start delivering new messages of a conversation. Replaces an existing feed
    /// for the same token, e.g. to change its options.
    #[asynced]
    pub fn subscribe(&self, token: String, options: ChatFeedOptions) -> Result<(), NcError> {
        let id = self.next_feed_id.get();
        self.next_feed_id.set(id + 1);

        let feed_token = token.clone();
        let task = ShardRc::spawn_owned(&ShardRc::try_from_ref(self)?, move |this| {
            run_feed(this, id, feed_token, options)
        })?;
        let feed = Feed { id, task };
        if let Some(previous) = self.feeds.borrow_mut().insert(token, feed) {
            previous.task.abort();
        }
        Ok(())
    }

    #[asynced]
    pub fn unsubscribe(&self, token: String) {
        if let Some(feed) = self.feeds.borrow_mut().remove(&token) {
            feed.task.abort();
        }
    }

    #[asynced]
    pub fn unsubscribe_all(&self) {
        for (_, feed) in self.feeds.borrow_mut().drain() {
            feed.task.abort();
        }
    }

    #[asynced]
    pub fn subscriptions(&self) -> Vec<String> {
        self.feeds.borrow().keys().cloned().collect()
    }

    fn stop_feed(&self, id: u64, token: String, error: NcError) {
        let mut feeds = self.feeds.borrow_mut();
        if feeds.get(&token).is_some_and(|feed| feed.id == id) {
            feeds.remove(&token);
        }
        drop(feeds);
        self.events
            .on_feed_stopped()
            .labelled(ConversationLabel::new(token.clone()))
            .emit(token, error);
    }
}

/// Long-poll loop. Holds the service weakly and only upgrades it between requests,
/// so dropping the service ends the loop.
async fn run_feed(
    this: ShardWeak<ChatFeedService>,
    id: u64,
    token: String,
    options: ChatFeedOptions,
) {
    let path = chat_path(&token, "");
    let label = ConversationLabel::new(token.clone());
    let mut query = ReceiveMessagesQuery {
        look_into_future: true,
        last_known_message_id: options.last_known_message_id,
        include_last_known: false,
        limit: PAGE_SIZE,
        timeout: LONG_POLL_SECONDS,
        last_common_read_id: None,
        thread_id: options.thread_id,
        set_read_marker: options.set_read_marker,
        mark_notifications_as_read: options.set_read_marker,
        no_status_update: options.no_status_update,
    };
    let mut retry = RETRY_MIN;

    loop {
        let Some(client) = this.upgrade().map(|service| service.client.clone()) else {
            return;
        };
        let result = client
            .get(&path)
            .query(&query)
            .timeout(REQUEST_TIMEOUT)
            .send_optional::<Vec<ChatMessage>>()
            .await;
        drop(client);
        let Some(service) = this.upgrade() else {
            return;
        };

        match result {
            Ok(response) => {
                retry = RETRY_MIN;
                let last_given = response.header_i64("X-Chat-Last-Given");
                let last_common_read = response.header_i64("X-Chat-Last-Common-Read");
                // 304: no new messages within the poll timeout.
                let messages = response.data.unwrap_or_default();

                if let Some(newest) = last_given.or_else(|| messages.iter().map(|m| m.id).max()) {
                    query.last_known_message_id = query.last_known_message_id.max(newest);
                }
                if let Some(read) = last_common_read
                    && query.last_common_read_id != Some(read)
                {
                    // Passing it back makes the server answer early when it changes.
                    query.last_common_read_id = Some(read);
                    service
                        .events
                        .on_read_marker()
                        .labelled(label.clone())
                        .emit(token.clone(), read);
                }
                if !messages.is_empty() {
                    service
                        .events
                        .on_messages()
                        .labelled(label.clone())
                        .emit(token.clone(), messages);
                }
            }
            Err(error) if error.is_transient() => {
                drop(service);
                tracing::debug!(%token, %error, ?retry, "chat feed request failed, retrying");
                tokio::time::sleep(retry).await;
                retry = (retry * 2).min(RETRY_MAX);
            }
            Err(error) => {
                tracing::warn!(%token, %error, "chat feed stopped");
                service.stop_feed(id, token, error);
                return;
            }
        }
    }
}
