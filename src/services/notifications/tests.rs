//! End-to-end tests of the notification services against a local mock server.

use std::{
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::Duration,
};

use eventful_rs::*;
use serde_json::json;

use super::{
    NotificationFeedOptions, NotificationServices, feed::*, notification::NotificationServiceAsync,
};
use crate::{
    dto::notifications::Notification,
    services::test_support::{MockResponse, MockServer, recv},
};

declare_shard!(TestListenerShard, runtime = std);

fn notification(id: i64) -> serde_json::Value {
    json!({"notification_id": id, "app": "spreed", "object_type": "chat", "object_id": "abc/1"})
}

#[tokio::test]
async fn list_uses_etags_and_empty_responses() {
    let server = MockServer::start(|request| match request.header("If-None-Match") {
        Some("\"v1\"") => MockResponse::empty(304),
        Some(_) => MockResponse::empty(204),
        None => MockResponse::ocs(200, json!([notification(1)])).with_header("ETag", "\"v1\""),
    });
    let services = NotificationServices::new(&server.client().await)
        .await
        .unwrap();
    let notifications = &services.notifications;

    let list = notifications.list(None).await.unwrap().unwrap();
    assert_eq!(list.notifications.len(), 1);
    assert_eq!(list.etag.as_deref(), Some("\"v1\""));

    assert!(notifications.list(list.etag).await.unwrap().is_none());

    let empty = notifications
        .list(Some("\"old\"".into()))
        .await
        .unwrap()
        .unwrap();
    assert!(empty.notifications.is_empty());
}

#[tokio::test]
async fn feed_reports_new_and_removed_notifications() {
    let polls = AtomicUsize::new(0);
    let server = MockServer::start(move |_| match polls.fetch_add(1, Ordering::SeqCst) {
        0 => MockResponse::ocs(200, json!([notification(2), notification(1)]))
            .with_header("ETag", "a"),
        1 => MockResponse::empty(304),
        _ => MockResponse::ocs(200, json!([notification(3), notification(2)]))
            .with_header("ETag", "b"),
    });
    let services = NotificationServices::new(&server.client().await)
        .await
        .unwrap();
    let feed = &services.feed;

    let (changed_tx, changed_rx) = mpsc::channel::<usize>();
    let changed_tx = Mutex::new(changed_tx);
    let _changed = feed
        .on_notifications_changed()
        .on_shard(&TestListenerShard::handle())
        .connect(move |notifications: Vec<Notification>| {
            let _ = changed_tx.lock().unwrap().send(notifications.len());
        })
        .scoped();
    let (new_tx, new_rx) = mpsc::channel::<Vec<i64>>();
    let new_tx = Mutex::new(new_tx);
    let _new = feed
        .on_new_notifications()
        .on_shard(&TestListenerShard::handle())
        .connect(move |notifications: Vec<Notification>| {
            let ids = notifications.iter().map(|n| n.notification_id).collect();
            let _ = new_tx.lock().unwrap().send(ids);
        })
        .scoped();
    let (removed_tx, removed_rx) = mpsc::channel::<Vec<i64>>();
    let removed_tx = Mutex::new(removed_tx);
    let _removed = feed
        .on_notifications_removed()
        .on_shard(&TestListenerShard::handle())
        .connect(move |ids| {
            let _ = removed_tx.lock().unwrap().send(ids);
        })
        .scoped();

    feed.start(NotificationFeedOptions {
        interval: Duration::from_millis(20),
    })
    .await
    .unwrap();

    let (first, changed_rx) = recv(changed_rx).await;
    assert_eq!(first, 2);
    // The baseline is not reported as new; only notification 3 is.
    assert_eq!(recv(new_rx).await.0, vec![3]);
    assert_eq!(recv(removed_rx).await.0, vec![1]);
    assert_eq!(recv(changed_rx).await.0, 2);
    feed.stop().await;

    let requests = server.requests();
    assert!(requests[0].header("If-None-Match").is_none());
    assert_eq!(requests[1].header("If-None-Match"), Some("a"));
    assert_eq!(requests[2].header("If-None-Match"), Some("a"));
}
