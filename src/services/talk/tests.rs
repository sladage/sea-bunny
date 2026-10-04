//! End-to-end tests of the Talk services against a local mock server.

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
    ConversationLabel, TalkServices,
    calls::call::CallServiceAsync,
    chat::{
        ChatFeedOptions, UploadAttachment, feed::*, messages::ChatServiceAsync,
        reactions::ReactionServiceAsync, sharing::ChatShareServiceAsync,
    },
    conversations::{ConversationFeedOptions, conversation::ConversationServiceAsync, feed::*},
};
use crate::{
    dto::talk::{
        AttachmentMetaData, CallNotificationState, ChatMessage, Conversation,
        ConversationListQuery, HistoryQuery,
    },
    services::{
        ncclient::NcError,
        test_support::{MockResponse, MockServer, recv},
    },
};

declare_shard!(TestListenerShard, runtime = std);

const TALK: &str = "/ocs/v2.php/apps/spreed/api";

#[tokio::test]
async fn services_reach_the_server_and_map_responses() {
    let server = MockServer::start(|request| match request.path() {
        p if p == format!("{TALK}/v4/room") => MockResponse::ocs(
            200,
            json!([{"token": "abc", "type": 2, "displayName": "Team", "lastMessage": []}]),
        )
        .with_header("X-Nextcloud-Talk-Modified-Before", 123),
        p if p == format!("{TALK}/v4/room/missing") => {
            MockResponse::ocs(404, json!({"error": "room"}))
        }
        p if p == format!("{TALK}/v1/chat/abc") => {
            MockResponse::ocs(200, json!([{"id": 10}, {"id": 9}]))
                .with_header("X-Chat-Last-Given", 9)
        }
        _ => MockResponse::empty(500),
    });
    let talk = TalkServices::new(&server.client().await).await.unwrap();

    let list = talk
        .conversations
        .conversations
        .list(ConversationListQuery::default())
        .await
        .unwrap();
    assert_eq!(list.conversations[0].display_name, "Team");
    assert_eq!(list.modified_before, Some(123));

    let error = talk
        .conversations
        .conversations
        .get("missing".into())
        .await
        .unwrap_err();
    assert!(
        matches!(&error, NcError::Api { status: 404, error: Some(e), .. } if e == "room"),
        "{error:?}"
    );

    let page = talk
        .chat
        .messages
        .history("abc".into(), HistoryQuery::default())
        .await
        .unwrap();
    assert_eq!(page.messages.len(), 2);
    assert_eq!(page.last_given, Some(9));

    let requests = server.requests();
    assert!(requests.iter().all(|r| r.header("authorization").is_some()));
    let history = requests
        .iter()
        .find(|r| r.path().ends_with("/chat/abc"))
        .unwrap();
    assert!(
        history.target.contains("lookIntoFuture=0"),
        "{}",
        history.target
    );
}

#[tokio::test]
async fn chat_feed_delivers_labelled_events_and_advances() {
    let polls = AtomicUsize::new(0);
    let server = MockServer::start(move |request| {
        if !request.path().ends_with("/chat/abc") {
            return MockResponse::empty(500);
        }
        if polls.fetch_add(1, Ordering::SeqCst) == 0 {
            MockResponse::ocs(200, json!([{"id": 11, "token": "abc", "message": "hello"}]))
                .with_header("X-Chat-Last-Given", 11)
                .with_header("X-Chat-Last-Common-Read", 10)
        } else {
            // The long poll expiring without news.
            std::thread::sleep(Duration::from_millis(50));
            MockResponse::empty(304)
        }
    });
    let talk = TalkServices::new(&server.client().await).await.unwrap();

    let (tx, rx) = mpsc::channel::<String>();
    let tx = Mutex::new(tx);
    let _messages = talk
        .chat
        .feed
        .on_messages()
        .labelled(ConversationLabel::new("abc"))
        .on_shard(&TestListenerShard::handle())
        .connect(move |token, messages: Vec<ChatMessage>| {
            let ids: Vec<_> = messages.iter().map(|m| m.id.to_string()).collect();
            let _ = tx
                .lock()
                .unwrap()
                .send(format!("{token}:{}", ids.join(",")));
        })
        .scoped();
    let (read_tx, read_rx) = mpsc::channel::<i64>();
    let read_tx = Mutex::new(read_tx);
    let _reads = talk
        .chat
        .feed
        .on_read_marker()
        .on_shard(&TestListenerShard::handle())
        .connect(move |_token, read| {
            let _ = read_tx.lock().unwrap().send(read);
        })
        .scoped();

    let options = ChatFeedOptions {
        last_known_message_id: 10,
        ..Default::default()
    };
    talk.chat
        .feed
        .subscribe("abc".into(), options)
        .await
        .unwrap();

    assert_eq!(recv(rx).await.0, "abc:11");
    assert_eq!(recv(read_rx).await.0, 10);

    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        talk.chat.feed.subscriptions().await,
        vec!["abc".to_string()]
    );
    talk.chat.feed.unsubscribe("abc".into()).await;
    assert!(talk.chat.feed.subscriptions().await.is_empty());

    let polls: Vec<_> = server
        .requests()
        .into_iter()
        .filter(|r| r.path().ends_with("/chat/abc"))
        .collect();
    assert!(polls.len() >= 2, "{polls:?}");
    assert!(polls[0].target.contains("lastKnownMessageId=10"));
    assert!(polls[1].target.contains("lastKnownMessageId=11"));
    assert!(polls[1].target.contains("lastCommonReadId=10"));
}

#[tokio::test]
async fn conversation_feed_reports_changes_and_removals() {
    let polls = AtomicUsize::new(0);
    let server = MockServer::start(move |_| {
        let rooms = match polls.fetch_add(1, Ordering::SeqCst) {
            0 => json!([{"token": "abc"}, {"token": "def"}]),
            1 => json!([{"token": "abc", "displayName": "Renamed"}]),
            _ => json!([{"token": "abc"}]),
        };
        MockResponse::ocs(200, rooms)
            .with_header("X-Nextcloud-Talk-Modified-Before", 100)
            .with_header("X-Nextcloud-Talk-Federation-Invites", 1)
    });
    let talk = TalkServices::new(&server.client().await).await.unwrap();
    let feed = &talk.conversations.feed;

    let (changed_tx, changed_rx) = mpsc::channel::<(Vec<String>, bool)>();
    let changed_tx = Mutex::new(changed_tx);
    let _changed = feed
        .on_conversations_changed()
        .on_shard(&TestListenerShard::handle())
        .connect(move |conversations: Vec<Conversation>, full| {
            let tokens = conversations.into_iter().map(|c| c.token).collect();
            let _ = changed_tx.lock().unwrap().send((tokens, full));
        })
        .scoped();
    let (removed_tx, removed_rx) = mpsc::channel::<Vec<String>>();
    let removed_tx = Mutex::new(removed_tx);
    let _removed = feed
        .on_conversations_removed()
        .on_shard(&TestListenerShard::handle())
        .connect(move |tokens| {
            let _ = removed_tx.lock().unwrap().send(tokens);
        })
        .scoped();
    let (invites_tx, invites_rx) = mpsc::channel::<i64>();
    let invites_tx = Mutex::new(invites_tx);
    let _invites = feed
        .on_federation_invites_changed()
        .on_shard(&TestListenerShard::handle())
        .connect(move |pending| {
            let _ = invites_tx.lock().unwrap().send(pending);
        })
        .scoped();

    feed.start(ConversationFeedOptions {
        interval: Duration::from_millis(20),
        full_refresh_every: 2,
        include_status: false,
    })
    .await
    .unwrap();

    let (first, changed_rx) = recv(changed_rx).await;
    assert_eq!(first, (vec!["abc".to_string(), "def".to_string()], true));
    let (second, _) = recv(changed_rx).await;
    assert_eq!(second, (vec!["abc".to_string()], false));
    assert_eq!(recv(removed_rx).await.0, vec!["def".to_string()]);
    assert_eq!(recv(invites_rx).await.0, 1);

    assert!(feed.is_running().await);
    feed.stop().await;
    assert!(!feed.is_running().await);

    let requests = server.requests();
    assert!(requests[0].target.contains("modifiedSince=0"));
    assert!(requests[1].target.contains("modifiedSince=100"));
    assert!(requests[2].target.contains("modifiedSince=0"));
    assert!(
        requests
            .iter()
            .all(|r| r.target.contains("noStatusUpdate=1"))
    );
}

#[tokio::test]
async fn upload_probes_folder_puts_file_and_posts_it() {
    let server = MockServer::start(|request| match (request.method.as_str(), request.path()) {
        ("POST", p) if p.ends_with("/chat/abc/attachment/folder") => MockResponse::ocs(
            200,
            json!({"folder": "/Talk/Team", "renames": [{"my file.txt": "my file (1).txt"}]}),
        ),
        ("PUT", _) => MockResponse::empty(201),
        ("POST", p) if p.ends_with("/chat/abc/attachment") => {
            MockResponse::ocs(200, json!({"renames": []}))
        }
        _ => MockResponse::empty(500),
    });
    let talk = TalkServices::new(&server.client().await).await.unwrap();

    let uploaded = talk
        .chat
        .sharing
        .upload(
            "abc".into(),
            UploadAttachment {
                file_name: "my file.txt".into(),
                data: b"hello".to_vec(),
                meta: AttachmentMetaData {
                    caption: Some("Look".into()),
                    ..Default::default()
                },
                reference_id: Some("ref-1".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(uploaded.file_path, "/Talk/Team/my file (1).txt");

    let requests = server.requests();
    assert_eq!(requests[0].body_json()["fileNames"], json!(["my file.txt"]));
    assert_eq!(
        requests[1].path(),
        "/remote.php/dav/files/alice/Talk/Team/my%20file%20%281%29.txt"
    );
    assert_eq!(requests[1].body, b"hello");
    let post = requests[2].body_json();
    assert_eq!(post["filePath"], "/Talk/Team/my file (1).txt");
    assert_eq!(post["referenceId"], "ref-1");
    assert_eq!(post["talkMetaData"], r#"{"caption":"Look"}"#);
}

#[tokio::test]
async fn call_notification_state_maps_statuses() {
    let server = MockServer::start(|request| {
        let status = match request.path().rsplit('/').nth(1) {
            Some("ringing") => 200,
            Some("missed") => 201,
            _ => 404,
        };
        MockResponse::ocs(status, json!(null))
    });
    let talk = TalkServices::new(&server.client().await).await.unwrap();
    let calls = &talk.calls.calls;

    assert_eq!(
        calls.notification_state("ringing".into()).await.unwrap(),
        CallNotificationState::KeepRinging
    );
    assert_eq!(
        calls.notification_state("missed".into()).await.unwrap(),
        CallNotificationState::Missed
    );
    assert_eq!(
        calls.notification_state("ended".into()).await.unwrap(),
        CallNotificationState::Dismiss
    );
}

#[tokio::test]
async fn php_empty_maps_deserialize_as_empty() {
    let server = MockServer::start(|_| MockResponse::ocs(200, json!([])));
    let talk = TalkServices::new(&server.client().await).await.unwrap();

    let reactions = talk
        .chat
        .reactions
        .list("abc".into(), 1, None)
        .await
        .unwrap();
    assert!(reactions.is_empty());
    let shared = talk.chat.sharing.overview("abc".into(), 7).await.unwrap();
    assert!(shared.is_empty());
}
