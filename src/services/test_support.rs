//! A minimal HTTP server for testing services end to end.

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
};

use eventful_rs::*;
use url::Url;

use crate::services::ncclient::NCClient;

#[derive(Debug, Clone)]
pub struct MockRequest {
    pub method: String,
    /// Path and query, e.g. `/ocs/v2.php/...?limit=10`.
    pub target: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl MockRequest {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub fn path(&self) -> &str {
        self.target.split('?').next().unwrap_or("")
    }

    pub fn body_json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body).unwrap_or_default()
    }
}

pub struct MockResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl MockResponse {
    /// An OCS response wrapping `data`.
    pub fn ocs(status: u16, data: serde_json::Value) -> Self {
        let body = serde_json::json!({
            "ocs": {"meta": {"status": "ok", "statuscode": status}, "data": data}
        });
        Self {
            status,
            headers: Vec::new(),
            body: body.to_string(),
        }
    }

    pub fn empty(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: String::new(),
        }
    }

    pub fn with_header(mut self, name: &str, value: impl ToString) -> Self {
        self.headers.push((name.to_owned(), value.to_string()));
        self
    }
}

pub struct MockServer {
    pub url: Url,
    requests: Arc<Mutex<Vec<MockRequest>>>,
}

impl MockServer {
    /// Serve every request with `handler` on a background thread.
    pub fn start<F>(handler: F) -> Self
    where
        F: Fn(&MockRequest) -> MockResponse + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = Url::parse(&format!("http://{}/", listener.local_addr().unwrap())).unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let log = requests.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { return };
                let Some(request) = read_request(&mut stream) else {
                    continue;
                };
                let response = handler(&request);
                log.lock().unwrap().push(request);
                let reason = match response.status {
                    200 => "OK",
                    201 => "Created",
                    204 => "No Content",
                    304 => "Not Modified",
                    404 => "Not Found",
                    _ => "Status",
                };
                let mut head = format!("HTTP/1.1 {} {reason}\r\n", response.status);
                for (name, value) in &response.headers {
                    head.push_str(&format!("{name}: {value}\r\n"));
                }
                let _ = write!(
                    stream,
                    "{head}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response.body.len(),
                    response.body
                );
            }
        });
        Self { url, requests }
    }

    pub fn requests(&self) -> Vec<MockRequest> {
        self.requests.lock().unwrap().clone()
    }

    /// A client logged in as `alice` against this server.
    pub async fn client(&self) -> ShardRcHandle<NCClient> {
        let client = NCClient::new(self.url.clone()).await.unwrap();
        client
            .deferred_upgrade_in_shard(async |client: &NCClient| {
                client.set_test_session("alice", "secret")
            })
            .await;
        client
    }
}

fn read_request(stream: &mut std::net::TcpStream) -> Option<MockRequest> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).ok()?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_owned();
    let target = parts.next()?.to_owned();

    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).ok()?;
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_owned(), value.trim().to_owned()));
        }
    }
    let length = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.parse().ok())
        .unwrap_or(0);
    let mut body = vec![0; length];
    reader.read_exact(&mut body).ok()?;
    Some(MockRequest {
        method,
        target,
        headers,
        body,
    })
}

/// Wait for a value from a channel without blocking the async runtime.
pub async fn recv<T: Send + 'static>(
    rx: std::sync::mpsc::Receiver<T>,
) -> (T, std::sync::mpsc::Receiver<T>) {
    tokio::task::spawn_blocking(move || {
        let value = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("timed out waiting for an event");
        (value, rx)
    })
    .await
    .unwrap()
}
