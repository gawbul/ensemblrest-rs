//! Tests for the mock HTTP server.

mod common;

use common::mock::{MockResponse, MockServer};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

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
