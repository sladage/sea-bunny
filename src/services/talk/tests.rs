//! End-to-end tests of the service wiring against a local mock server.

use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

use eventful_rs::*;
use url::Url;

use super::{
    ConversationLabel, TalkServices, chat::ChatServiceAsync, chat_feed::*,
    conversations::ConversationServiceAsync,
};
use crate::{
    dto::talk::{ConversationListQuery, HistoryQuery},
    services::ncclient::{NCClient, NcError},
};

declare_shard!(TestListenerShard, runtime = std);

const CHAT: &str = "/ocs/v2.php/apps/spreed/api/v1/chat/abc";

/// Serves canned responses and records request lines with their auth header.
fn mock_server() -> (Url, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = Url::parse(&format!("http://{}/", listener.local_addr().unwrap())).unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let log = requests.clone();
    std::thread::spawn(move || {
        let mut feed_polls = 0;
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();
            let mut authorized = false;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line
                    .to_ascii_lowercase()
                    .starts_with("authorization: basic")
                {
                    authorized = true;
                }
                if line == "\r\n" || line.is_empty() {
                    break;
                }
            }
            let target = request_line
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .to_owned();
            log.lock().unwrap().push(format!(
                "{} {target} auth={authorized}",
                request_line.split(' ').next().unwrap()
            ));

            let (status, headers, body) = if target
                .starts_with("/ocs/v2.php/apps/spreed/api/v4/room?")
            {
                (
                    "200 OK",
                    "X-Nextcloud-Talk-Modified-Before: 123\r\n",
                    r#"{"ocs":{"meta":{"status":"ok","statuscode":200},"data":[{"token":"abc","type":2,"displayName":"Team","lastMessage":[]}]}}"#,
                )
            } else if target.starts_with("/ocs/v2.php/apps/spreed/api/v4/room/missing") {
                (
                    "404 Not Found",
                    "",
                    r#"{"ocs":{"meta":{"status":"failure","statuscode":404,"message":""},"data":{"error":"room"}}}"#,
                )
            } else if target.starts_with(CHAT) && target.contains("lookIntoFuture=0") {
                (
                    "200 OK",
                    "X-Chat-Last-Given: 9\r\n",
                    r#"{"ocs":{"meta":{"status":"ok","statuscode":200},"data":[{"id":10,"message":"older"},{"id":9,"message":"oldest"}]}}"#,
                )
            } else if target.starts_with(CHAT) {
                feed_polls += 1;
                if feed_polls == 1 {
                    (
                        "200 OK",
                        "X-Chat-Last-Given: 11\r\nX-Chat-Last-Common-Read: 10\r\n",
                        r#"{"ocs":{"meta":{"status":"ok","statuscode":200},"data":[{"id":11,"token":"abc","message":"hello"}]}}"#,
                    )
                } else {
                    // Simulate the long poll expiring without news.
                    std::thread::sleep(Duration::from_millis(50));
                    ("304 Not Modified", "", "")
                }
            } else {
                ("500 Internal Server Error", "", "")
            };
            let _ = write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    (url, requests)
}

async fn authenticated_services(url: Url) -> (ShardRcHandle<NCClient>, TalkServices) {
    let client = NCClient::new(url).await.unwrap();
    client
        .deferred_upgrade_in_shard(async |client: &NCClient| {
            client.set_test_credentials("alice", "secret")
        })
        .await;
    let talk = TalkServices::new(&client).await.unwrap();
    (client, talk)
}

#[tokio::test]
async fn services_reach_the_server_and_map_responses() {
    let (url, requests) = mock_server();
    let (_client, talk) = authenticated_services(url).await;

    let list = talk
        .conversations
        .list(ConversationListQuery::default())
        .await
        .unwrap();
    assert_eq!(list.conversations.len(), 1);
    assert_eq!(list.conversations[0].display_name, "Team");
    assert_eq!(list.modified_before, Some(123));

    let error = talk.conversations.get("missing".into()).await.unwrap_err();
    assert!(
        matches!(&error, NcError::Api { status: 404, error: Some(e), .. } if e == "room"),
        "{error:?}"
    );

    let page = talk
        .chat
        .history("abc".into(), HistoryQuery::default())
        .await
        .unwrap();
    assert_eq!(page.messages.len(), 2);
    assert_eq!(page.last_given, Some(9));

    let requests = requests.lock().unwrap();
    assert!(
        requests.iter().all(|r| r.ends_with("auth=true")),
        "{requests:?}"
    );
}

#[tokio::test]
async fn chat_feed_delivers_labelled_events_and_advances() {
    let (url, requests) = mock_server();
    let (_client, talk) = authenticated_services(url).await;

    let (tx, rx) = mpsc::channel::<String>();
    let tx = Mutex::new(tx);
    let _messages = talk
        .chat_feed
        .on_messages()
        .labelled(ConversationLabel::new("abc"))
        .on_shard(&TestListenerShard::handle())
        .connect(move |token, messages: Vec<crate::dto::talk::ChatMessage>| {
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
        .chat_feed
        .on_read_marker()
        .on_shard(&TestListenerShard::handle())
        .connect(move |_token, read| {
            let _ = read_tx.lock().unwrap().send(read);
        })
        .scoped();

    talk.chat_feed
        .subscribe(
            "abc".into(),
            ChatFeedOptions {
                last_known_message_id: 10,
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let received = tokio::task::spawn_blocking(move || rx.recv_timeout(Duration::from_secs(5)))
        .await
        .unwrap()
        .expect("no messages delivered");
    assert_eq!(received, "abc:11");
    let read = tokio::task::spawn_blocking(move || read_rx.recv_timeout(Duration::from_secs(5)))
        .await
        .unwrap()
        .expect("no read marker delivered");
    assert_eq!(read, 10);

    // Let the feed poll again, then check it continues after the newest message.
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        talk.chat_feed.subscriptions().await,
        vec!["abc".to_string()]
    );
    talk.chat_feed.unsubscribe("abc".into()).await;
    assert!(talk.chat_feed.subscriptions().await.is_empty());

    let requests = requests.lock().unwrap();
    let polls: Vec<_> = requests.iter().filter(|r| r.contains(CHAT)).collect();
    assert!(polls[0].contains("lastKnownMessageId=10"), "{polls:?}");
    assert!(polls.len() >= 2, "{polls:?}");
    assert!(polls[1].contains("lastKnownMessageId=11"), "{polls:?}");
    assert!(polls[1].contains("lastCommonReadId=10"), "{polls:?}");
}
