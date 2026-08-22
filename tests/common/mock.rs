//! A minimal HTTP/1.1 server for testing, built only on `std`.
//!
//! This is the equivalent of Go's `httptest.Server`. It speaks plain HTTP, so no
//! TLS is involved, and it answers each connection from a scripted queue while
//! recording what it received.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// A scripted response for the mock server to return.
#[derive(Debug, Clone)]
pub struct MockResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl MockResponse {
    /// A JSON response.
    pub fn json(status: u16, body: &str) -> Self {
        Self {
            status,
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: body.as_bytes().to_vec(),
        }
    }

    /// A response with an arbitrary content type.
    pub fn text(status: u16, content_type: &str, body: &str) -> Self {
        Self {
            status,
            headers: vec![("Content-Type".into(), content_type.into())],
            body: body.as_bytes().to_vec(),
        }
    }

    /// Adds a response header, for example rate-limit telemetry.
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }
}

/// A request the mock server received.
#[derive(Debug, Clone)]
pub struct RecordedRequest {
    /// The HTTP method, uppercase.
    pub method: String,
    /// The full request target, including any query string.
    pub target: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl RecordedRequest {
    /// The request path, excluding the query string.
    pub fn path(&self) -> &str {
        self.target.split('?').next().unwrap_or(&self.target)
    }

    /// The raw query string, or `""` when there was none.
    pub fn query(&self) -> &str {
        self.target.split_once('?').map_or("", |(_, q)| q)
    }

    /// Looks up a header case-insensitively.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Parses the request body as JSON.
    pub fn json(&self) -> ensemblrest::serde_json::Value {
        ensemblrest::serde_json::from_slice(&self.body)
            .unwrap_or(ensemblrest::serde_json::Value::Null)
    }
}

/// A single-threaded HTTP server answering from a scripted queue.
pub struct MockServer {
    base_url: String,
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    timeout: Duration,
}

impl MockServer {
    /// Starts a server that answers requests from `responses` in order.
    ///
    /// Once the queue is exhausted every further request gets a 500. Scripting
    /// one response per expected attempt is how the retry tests assert attempt
    /// counts.
    ///
    /// Uses a default 5-second I/O timeout per request. To use a different timeout
    /// (e.g., for tests with stalled clients), use `start_with_timeout`.
    pub fn start(responses: Vec<MockResponse>) -> Self {
        Self::start_with_timeout(responses, Duration::from_secs(5))
    }

    /// Starts a server with a custom I/O timeout per request.
    ///
    /// The timeout applies to both read and write operations on accepted streams.
    /// When a timeout occurs, the connection is dropped without recording the request.
    ///
    /// The default 5-second timeout is suitable for production use and prevents hangs.
    /// Tests with intentionally stalled clients should use a short timeout (e.g., 200ms)
    /// to complete quickly.
    pub fn start_with_timeout(responses: Vec<MockResponse>, timeout: Duration) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));

        let thread_requests = Arc::clone(&requests);
        let thread_stop = Arc::clone(&stop);
        let mut queue: VecDeque<MockResponse> = responses.into();

        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                if thread_stop.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(stream) = stream else { break };
                let scripted = queue.pop_front();
                if let Ok(req) = serve_one(stream, scripted, timeout) {
                    thread_requests.lock().expect("requests lock").push(req);
                }
            }
        });

        Self {
            base_url: format!("http://{addr}"),
            addr,
            requests,
            stop,
            handle: Some(handle),
            timeout,
        }
    }

    /// Convenience for a server returning a single JSON response.
    pub fn with_json(status: u16, body: &str) -> Self {
        Self::start(vec![MockResponse::json(status, body)])
    }

    /// The base URL to hand to `Client::builder().base_url(..)`.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Every request received so far.
    pub fn requests(&self) -> Vec<RecordedRequest> {
        self.requests.lock().expect("requests lock").clone()
    }

    /// The number of requests received so far.
    pub fn request_count(&self) -> usize {
        self.requests.lock().expect("requests lock").len()
    }

    /// The single request received, panicking if there was not exactly one.
    pub fn only_request(&self) -> RecordedRequest {
        let reqs = self.requests();
        assert_eq!(
            reqs.len(),
            1,
            "expected exactly one request, got {}",
            reqs.len()
        );
        reqs.into_iter().next().expect("one request")
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // Unblock the accept loop so the thread can observe the stop flag.
        let _ = TcpStream::connect(self.addr);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn serve_one(
    mut stream: TcpStream,
    scripted: Option<MockResponse>,
    timeout: Duration,
) -> std::io::Result<RecordedRequest> {
    // Set read and write timeouts to prevent hanging on stalled clients. This converts
    // a silent hang into a fast failure, and ensures MockServer::drop's join() can
    // complete even if serve_one is mid-request.
    let _ = timeout;
    let _ = timeout;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;

    let mut reader = BufReader::new(stream.try_clone()?);

    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let target = parts.next().unwrap_or_default().to_string();

    let mut headers = Vec::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            let (k, v) = (k.trim().to_string(), v.trim().to_string());
            if k.eq_ignore_ascii_case("content-length") {
                content_length = v.parse().unwrap_or(0);
            }
            headers.push((k, v));
        }
    }

    let mut body = vec![0u8; content_length];
    let body_read_failed = if content_length > 0 {
        reader.read_exact(&mut body).is_err()
    } else {
        false
    };

    // If body read timed out or failed, send a 408 timeout response.
    // This proves the timeout mechanism works: the server sent a response
    // only because the timeout fired, not because the stream dropped.
    let response = if body_read_failed {
        MockResponse {
            status: 408,
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: br#"{"error":"request timeout"}"#.to_vec(),
        }
    } else {
        scripted.unwrap_or_else(|| MockResponse {
            status: 500,
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: br#"{"error":"mock server: no scripted response left"}"#.to_vec(),
        })
    };

    let mut head = format!(
        "HTTP/1.1 {} {}\r\n",
        response.status,
        reason(response.status)
    );
    for (k, v) in &response.headers {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    head.push_str(&format!("Content-Length: {}\r\n", response.body.len()));
    // Closing each connection keeps the server free of keep-alive bookkeeping.
    head.push_str("Connection: close\r\n\r\n");

    stream.write_all(head.as_bytes())?;
    stream.write_all(&response.body)?;
    stream.flush()?;

    // If body read timed out, return an error so the request is not recorded.
    // The response was still sent, proving the timeout worked.
    if body_read_failed {
        return Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "request body read timeout",
        ));
    }

    Ok(RecordedRequest {
        method,
        target,
        headers,
        body,
    })
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        408 => "Request Timeout",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "Unknown",
    }
}
