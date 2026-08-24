//! Tests for the mock HTTP server.

mod common;

use common::mock::{MockResponse, MockServer};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Sends a raw HTTP/1.1 request and returns the status line plus body.
fn raw_request(base_url: &str, request: &str) -> String {
    let addr = base_url.trim_start_matches("http://");
    let mut stream = TcpStream::connect(addr).expect("connect");
    stream.write_all(request.as_bytes()).expect("write");
    stream.flush().expect("flush");
    let mut out = String::new();
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    while reader.read_line(&mut line).expect("read") > 0 {
        out.push_str(&line);
        line.clear();
    }
    out
}

#[test]
fn serves_a_scripted_response_and_records_the_request() {
    let server = MockServer::with_json(200, r#"{"ping":1}"#);
    let raw = raw_request(
        server.base_url(),
        "GET /info/ping?x=1 HTTP/1.1\r\nHost: localhost\r\nX-Trace: abc\r\n\r\n",
    );

    assert!(raw.starts_with("HTTP/1.1 200 OK"), "got {raw}");
    assert!(raw.contains(r#"{"ping":1}"#), "got {raw}");

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/ping");
    assert_eq!(req.query(), "x=1");
    assert_eq!(
        req.header("x-trace"),
        Some("abc"),
        "header lookup is case-insensitive"
    );
}

#[test]
fn reads_a_request_body_using_content_length() {
    let server = MockServer::with_json(200, "{}");
    raw_request(
        server.base_url(),
        "POST /lookup/id HTTP/1.1\r\nHost: localhost\r\nContent-Length: 18\r\n\r\n{\"ids\":[\"ENSG01\"]}",
    );

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.json()["ids"][0], "ENSG01");
}

#[test]
fn exhausted_queue_returns_500() {
    let server = MockServer::start(vec![MockResponse::json(200, "{}")]);
    raw_request(
        server.base_url(),
        "GET /a HTTP/1.1\r\nHost: localhost\r\n\r\n",
    );
    let raw = raw_request(
        server.base_url(),
        "GET /b HTTP/1.1\r\nHost: localhost\r\n\r\n",
    );
    assert!(raw.starts_with("HTTP/1.1 500"), "got {raw}");
    assert_eq!(server.request_count(), 2);
}

#[test]
fn read_timeout_prevents_hanging_on_incomplete_requests() {
    use std::thread as std_thread;

    let server = MockServer::start_with_timeout(
        vec![MockResponse::json(200, "{}")],
        Duration::from_millis(300),
    );

    let base_url = server.base_url().to_string();

    // Spawn a thread that connects, sends incomplete data, and tries to read the response.
    // If server has timeout: gets 408 response quickly (~300ms)
    // If server has no timeout: read times out after we set a read timeout on our end
    let handle = std_thread::spawn(move || {
        let addr = base_url.trim_start_matches("http://");
        if let Ok(mut stream) = TcpStream::connect(addr) {
            // Send a request claiming Content-Length: 100 but only send 10 bytes.
            let incomplete =
                b"POST /test HTTP/1.1\r\nHost: localhost\r\nContent-Length: 100\r\n\r\n0123456789";
            let _ = stream.write_all(incomplete);
            let _ = stream.flush();

            // Try to read the response with a 1-second timeout.
            // If server timed out and sent 408: we get the response immediately (~300ms)
            // If server has no timeout and is hung: read times out after 1s
            let _ = stream.set_read_timeout(Some(Duration::from_millis(1000)));
            let mut buf = [0u8; 1024];
            match stream.read(&mut buf) {
                Ok(n) => {
                    // Got response. Check if it contains "408" (timeout response)
                    let response = std::str::from_utf8(&buf[..n]).unwrap_or("");
                    if response.contains("408") {
                        "got_408".to_string()
                    } else {
                        "got_other".to_string()
                    }
                }
                Err(_) => {
                    // Read timed out or failed. If no server timeout, our 1s read timeout fired.
                    "read_timeout".to_string()
                }
            }
        } else {
            panic!("failed to connect");
        }
    });

    // Join with a bound above the read timeout (1.5s is safe, server response is ~300ms).
    let timeout = Duration::from_millis(1500);
    let max_wait = std::time::Instant::now() + timeout;
    loop {
        if handle.is_finished() {
            break;
        }
        if std::time::Instant::now() > max_wait {
            panic!("client thread did not finish within {timeout:?}; server likely hangs forever");
        }
        std_thread::sleep(Duration::from_millis(50));
    }

    // Thread finished. Check result.
    let result = handle.join().expect("client thread panicked");
    match result.as_str() {
        "got_408" => {
            // Success! Server sent 408 timeout response, proving timeout worked.
        }
        "got_other" => {
            panic!("server sent response other than 408, expected timeout response");
        }
        "read_timeout" => {
            panic!("our read timed out; server likely had no timeout and hung forever");
        }
        _ => panic!("unexpected result: {}", result),
    }

    // Request should not be recorded because body read timed out on server.
    assert_eq!(
        server.request_count(),
        0,
        "incomplete request should not be recorded"
    );
}
