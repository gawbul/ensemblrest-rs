//! Integration tests for request execution, retries and backoff.

mod common;

use common::mock::{MockResponse, MockServer};
use ensemblrest::endpoints::Method;
use ensemblrest::options::{content_type, header, query};
use ensemblrest::serde_json;
use ensemblrest::{ApiErrorKind, Client, Error};
use std::time::Duration;

/// A client pointed at `server` with backoff short enough for a fast test.
fn client(server: &MockServer, max_attempts: u32) -> Client {
    Client::builder()
        .base_url(server.base_url())
        .max_attempts(max_attempts)
        .wall_time_for_test()
        .build()
        .unwrap()
}

#[test]
fn successful_get_returns_the_body() {
    let server = MockServer::with_json(200, r#"{"ping":1}"#);
    let c = client(&server, 5);
    let resp = c
        .call_raw(Method::Get, "/info/ping", &[], None, &[])
        .unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(resp.json::<serde_json::Value>().unwrap()["ping"], 1);
    assert_eq!(server.only_request().path(), "/info/ping");
}

#[test]
fn path_parameters_are_substituted_and_colons_preserved() {
    let server = MockServer::with_json(200, "{}");
    let c = client(&server, 5);
    c.call_raw(
        Method::Get,
        "/sequence/region/{{species}}/{{region}}",
        &[("species", "homo_sapiens"), ("region", "X:1000..2000:1")],
        None,
        &[],
    )
    .unwrap();

    assert_eq!(
        server.only_request().path(),
        "/sequence/region/homo_sapiens/X:1000..2000:1",
        "colons must not be percent-encoded"
    );
}

#[test]
fn query_options_reach_the_wire_sorted() {
    let server = MockServer::with_json(200, "{}");
    let c = client(&server, 5);
    c.call_raw(
        Method::Get,
        "/lookup/id/{{id}}",
        &[("id", "ENSG01")],
        None,
        &[query("expand", "1"), query("utf8", "1")],
    )
    .unwrap();

    assert_eq!(server.only_request().query(), "expand=1&utf8=1");
}

#[test]
fn headers_and_content_type_are_applied() {
    let server = MockServer::with_json(200, "{}");
    let c = Client::builder()
        .base_url(server.base_url())
        .user_agent("test-agent/9")
        .header("X-Client", "persistent")
        .build()
        .unwrap();

    c.call_raw(
        Method::Get,
        "/info/ping",
        &[],
        None,
        &[
            content_type("text/x-fasta"),
            header("X-Call", "per-request"),
        ],
    )
    .unwrap();

    let req = server.only_request();
    assert_eq!(req.header("user-agent"), Some("test-agent/9"));
    assert_eq!(req.header("x-client"), Some("persistent"));
    assert_eq!(req.header("x-call"), Some("per-request"));
    assert_eq!(req.header("accept"), Some("text/x-fasta"));
    assert_eq!(req.header("content-type"), Some("text/x-fasta"));
}

#[test]
fn conflicting_headers_replace_rather_than_accumulate() {
    // `ureq`'s `.header()` appends to the underlying HeaderMap; without an
    // explicit replace step, a caller-supplied header of the same name as one
    // of the fixed headers would sit alongside it on the wire instead of
    // losing, and `User-Agent` is a singleton per RFC 9110 so recipients take
    // the FIRST value — meaning an unreplaced duplicate would let the wrong
    // side win silently.
    let server = MockServer::with_json(200, "{}");
    let c = Client::builder()
        .base_url(server.base_url())
        .user_agent("client-ua")
        .header("Accept", "application/json")
        .build()
        .unwrap();

    c.call_raw(
        Method::Get,
        "/sequence/id/{{id}}",
        &[("id", "ENSG01")],
        None,
        &[
            header("User-Agent", "evil-ua"),
            content_type("text/x-fasta"),
        ],
    )
    .unwrap();

    let req = server.only_request();

    // The client's configured User-Agent must win over a per-request override.
    assert_eq!(req.header("user-agent"), Some("client-ua"));
    assert_eq!(
        req.headers
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case("user-agent"))
            .count(),
        1,
        "exactly one User-Agent header must reach the wire, got {:?}",
        req.headers
    );

    // content_type(...) must fully replace a client-level Accept, not merge
    // with it into a comma-joined value the server could misinterpret.
    assert_eq!(req.header("accept"), Some("text/x-fasta"));
    assert_eq!(
        req.headers
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case("accept"))
            .count(),
        1,
        "exactly one Accept header must reach the wire, got {:?}",
        req.headers
    );
}

#[test]
fn retries_do_not_bypass_the_rate_limiter() {
    // `wall_time_for_test()` sets a window so wide (1000 reqs / 5ms) that the
    // limiter never throttles, so no other test in this file would fail if
    // `limiter.wait()` were hoisted out of the retry loop. Pin it directly
    // with a real one-request-per-window limit instead.
    //
    // `rate_limit()` sets both the limiter's window AND the backoff base
    // (they're the same `wallTime` field in the Go port), so naively using
    // `rate_limit(1, Duration::from_millis(60))` alone would make backoff's
    // `wall_time * 2 * attempt` sleep 120ms on its own -- enough to clear a
    // ">= 60ms" assertion with the limiter contributing nothing observable.
    // `backoff_wall_time_for_test` decouples them: the limiter keeps its real
    // 300ms window, while the backoff base shrinks to near-zero, so the
    // limiter is the only thing that can produce the observed delay.
    let server = MockServer::start(vec![
        MockResponse::json(503, r#"{"error":"down"}"#),
        MockResponse::json(200, "{}"),
    ]);
    let c = Client::builder()
        .base_url(server.base_url())
        .max_attempts(5)
        .rate_limit(1, Duration::from_millis(300))
        .backoff_wall_time_for_test(Duration::from_millis(1))
        .build()
        .unwrap();

    let start = std::time::Instant::now();
    let resp = c.call_raw(Method::Get, "/a", &[], None, &[]).unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(server.request_count(), 2);
    assert!(
        start.elapsed() >= Duration::from_millis(300),
        "the retried attempt must still wait on the rate limiter, elapsed {:?}",
        start.elapsed()
    );
}

#[test]
fn post_sends_a_json_body() {
    let server = MockServer::with_json(200, "[]");
    let c = client(&server, 5);
    let body = serde_json::json!({ "ids": ["ENSG01", "ENSG02"] });
    c.call_raw(Method::Post, "/lookup/id", &[], Some(&body), &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.json()["ids"][1], "ENSG02");
}

#[test]
fn a_404_is_returned_immediately_without_retrying() {
    let server = MockServer::start(vec![MockResponse::json(404, r#"{"error":"not found"}"#)]);
    let c = client(&server, 5);
    let err = c
        .call_raw(
            Method::Get,
            "/lookup/id/{{id}}",
            &[("id", "NOPE")],
            None,
            &[],
        )
        .unwrap_err();

    assert_eq!(err.api_kind(), Some(ApiErrorKind::NotFound));
    assert!(err.is_not_found());
    assert_eq!(server.request_count(), 1, "404 must not be retried");
}

#[test]
fn a_503_is_retried_then_succeeds() {
    let server = MockServer::start(vec![
        MockResponse::json(503, r#"{"error":"down"}"#),
        MockResponse::json(200, r#"{"ping":1}"#),
    ]);
    let c = client(&server, 5);
    let resp = c
        .call_raw(Method::Get, "/info/ping", &[], None, &[])
        .unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(server.request_count(), 2);
}

#[test]
fn a_transient_400_is_retried_but_an_ordinary_400_is_not() {
    let transient = MockServer::start(vec![
        MockResponse::json(400, r#"{"error":"something bad has happened"}"#),
        MockResponse::json(200, "{}"),
    ]);
    client(&transient, 5)
        .call_raw(Method::Get, "/a", &[], None, &[])
        .unwrap();
    assert_eq!(transient.request_count(), 2);

    let fatal = MockServer::start(vec![MockResponse::json(400, r#"{"error":"bad id"}"#)]);
    let err = client(&fatal, 5)
        .call_raw(Method::Get, "/a", &[], None, &[])
        .unwrap_err();
    assert!(err.is_bad_request());
    assert_eq!(fatal.request_count(), 1);
}

#[test]
fn a_429_is_retried_honouring_retry_after() {
    let server = MockServer::start(vec![
        MockResponse::json(429, r#"{"error":"slow down"}"#).with_header("Retry-After", "0.05"),
        MockResponse::json(200, "{}"),
    ]);
    let c = client(&server, 5);
    let start = std::time::Instant::now();
    c.call_raw(Method::Get, "/a", &[], None, &[]).unwrap();

    assert_eq!(server.request_count(), 2);
    assert!(
        start.elapsed() >= Duration::from_millis(50),
        "Retry-After must be honoured"
    );
}

#[test]
fn exhausting_attempts_yields_max_retries_carrying_the_last_error() {
    let server = MockServer::start(vec![
        MockResponse::json(503, r#"{"error":"down"}"#),
        MockResponse::json(503, r#"{"error":"down"}"#),
    ]);
    let c = client(&server, 2);
    let err = c.call_raw(Method::Get, "/a", &[], None, &[]).unwrap_err();

    assert!(
        matches!(err, Error::MaxRetries { attempts: 2, .. }),
        "got {err:?}"
    );
    assert_eq!(err.api_kind(), Some(ApiErrorKind::ServiceUnavailable));
    assert_eq!(server.request_count(), 2);
}

#[test]
fn rate_limit_headers_are_captured_on_the_response() {
    let server = MockServer::start(vec![
        MockResponse::json(200, "{}")
            .with_header("X-RateLimit-Limit", "55000")
            .with_header("X-RateLimit-Remaining", "54999")
            .with_header("X-RateLimit-Period", "3600"),
    ]);
    let c = client(&server, 5);
    let resp = c.call_raw(Method::Get, "/a", &[], None, &[]).unwrap();

    assert_eq!(resp.rate_limit().limit, Some(55000));
    assert_eq!(resp.rate_limit().remaining, Some(54999));
    assert_eq!(c.rate_limit().period, Some(3600));
}

#[test]
fn a_missing_path_parameter_fails_before_any_request() {
    let server = MockServer::with_json(200, "{}");
    let c = client(&server, 5);
    let err = c
        .call_raw(Method::Get, "/lookup/id/{{id}}", &[], None, &[])
        .unwrap_err();

    assert!(
        matches!(&err, Error::MissingParam(n) if n == "id"),
        "got {err:?}"
    );
    assert_eq!(server.request_count(), 0, "must not hit the network");
}

#[test]
fn non_json_bodies_come_back_through_text() {
    let server = MockServer::start(vec![MockResponse::text(
        200,
        "text/x-fasta",
        ">ENSG00000157764\nACGT\n",
    )]);
    let c = client(&server, 5);
    let resp = c
        .call_raw(
            Method::Get,
            "/sequence/id/{{id}}",
            &[("id", "ENSG01")],
            None,
            &[content_type("text/x-fasta")],
        )
        .unwrap();

    assert_eq!(resp.text().unwrap(), ">ENSG00000157764\nACGT\n");
    assert_eq!(resp.content_type(), Some("text/x-fasta"));
}
