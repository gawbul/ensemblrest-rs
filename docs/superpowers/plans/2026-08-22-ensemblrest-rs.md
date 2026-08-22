# ensemblrest-rs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `ensemblrest`, a blocking Rust client library for the Ensembl REST API with all 106 endpoints, ported from `goensemblrest`.

**Architecture:** A `Client` wrapping `Arc<Inner>` (immutable config + a shared sliding-window rate limiter) drives a retry loop in `request.rs`. A static 106-entry `ENDPOINTS` table maps camelCase endpoint names to path templates; 16 domain modules add one thin typed method per endpoint, each delegating to `Client::call`. Every method returns `Result<Response>`, and the caller decodes with `.json::<T>()`, `.text()` or `.bytes()`.

**Tech Stack:** Rust 1.98 (edition 2024), `ureq` 3.4 (blocking HTTP + rustls TLS), `serde` 1.0, `serde_json` 1.0. Zero dev-dependencies — the test HTTP server is hand-written on `std::net::TcpListener`.

**Spec:** `docs/superpowers/specs/2026-08-22-ensemblrest-rs-design.md`

**Reference port:** `/Users/gawbul/Documents/Code/goensemblrest` — the Go source is the authority for endpoint data and behaviour. Read it directly when a task says to.

## Global Constraints

Every task's requirements implicitly include all of these.

- **Rust 1.98.0**, `edition = "2024"`, `rust-version = "1.98"`. Run `rustup update stable` before Task 1.
- **Exactly three direct dependencies:** `ureq = "3.4"`, `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`. Adding any fourth is a plan violation — including `regex`, `url`, `percent-encoding`, `http`, `lazy_static` and `once_cell`. Use `ureq::http::*` for `HeaderMap`/`StatusCode` (re-exported, not a direct dependency) and `std::sync::OnceLock` for lazy statics.
- **Zero dev-dependencies.** `[dev-dependencies]` must stay empty.
- **Endpoint table keys stay camelCase** (`"getLookupById"`). Never re-case them; they are the cross-port contract shared with `goensemblrest` and `pyEnsemblRest`.
- **Method names are snake_case** of the Go names: `GetLookupByID` → `get_lookup_by_id`, `GetLookupByMultipleIDs` → `get_lookup_by_multiple_ids`.
- **Colons must survive path encoding.** `13:32889611..32973805:1` must never become `13%3A32889611..32973805%3A1`.
- **Error `Display` strings are byte-identical to the Go and Python ports.** Exact format: `EnsEMBL REST API returned a 404 (Not Found): <message>`.
- **License header/copyright:** MIT, `Copyright (c) 2020-2026 Steve Moss`.
- Every task ends with `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test` passing before the commit.

---

## File Structure

| File | Responsibility | Task |
|---|---|---|
| `Cargo.toml` | Manifest: 3 deps, edition 2024, rust-version 1.98 | 1 |
| `LICENSE` | MIT, 2020-2026 Steve Moss | 1 |
| `src/lib.rs` | Crate docs, module declarations, re-exports, `DEFAULT_*` consts | 1 |
| `src/error.rs` | `Error`, `ApiError`, `ApiErrorKind`, status table, message parsing | 3 |
| `src/ratelimit.rs` | `RateLimitInfo`, `RateLimiter` (sliding window + header telemetry) | 2 |
| `src/encoding.rs` | Percent-encoding, `resolve_path`, query-string building | 4 |
| `src/response.rs` | `Response` with `.json()` / `.text()` / `.bytes()` | 5 |
| `src/options.rs` | `RequestOption`, `RequestConfig` resolution | 6 |
| `src/client.rs` | `Client`, `ClientBuilder`, `Inner` | 7 |
| `tests/common/mock.rs` | std-only HTTP/1.1 mock server | 8 |
| `src/request.rs` | Retry loop, transient classification, backoff, `execute` | 9 |
| `src/endpoints.rs` | `Method`, `EndpointSpec`, `ENDPOINTS` (106), index, `call`, `endpoints()` | 10 |
| `src/types.rs` | serde domain models | 11 |
| `src/{archive,lookup,sequence,xrefs,mapping,overlap,comparative}.rs` | Domain methods (group 1, 29 methods) | 12 |
| `src/{info,ontology,ld,regulation,transcript}.rs` | Domain methods (group 2, 39 methods) | 13 |
| `src/{ga4gh,variation,vep,phenotype}.rs` | Domain methods (group 3, 38 methods) | 14 |
| `tests/endpoints.rs` | Per-endpoint mock tests + 106↔106 parity test | 12-15 |
| `tests/live.rs` | Live smoke tests, `#[ignore]` + env-gated | 16 |
| `examples/basic.rs` | Runnable demonstration | 16 |
| `README.md`, `Makefile`, `AGENTS.md`, `CLAUDE.md` | Docs and automation | 17 |
| `.github/workflows/*.yaml` | CI: PR, nightly drift, tag release | 18 |

`src/encoding.rs` is split out from the spec's `request.rs` because encoding is pure and
independently testable, while `request.rs` needs a live socket. Keeping them separate
means Task 4 can be fully tested before any HTTP code exists.

---

### Task 1: Project scaffold

**Files:**
- Create: `Cargo.toml`, `src/lib.rs`, `LICENSE`, `.gitignore`
- Test: `src/lib.rs` (inline `#[cfg(test)]` module)

**Interfaces:**
- Consumes: nothing
- Produces: `ensemblrest::VERSION: &str`, `DEFAULT_BASE_URL: &str`, `DEFAULT_CONTENT_TYPE: &str`, `DEFAULT_TIMEOUT: Duration`, `DEFAULT_MAX_ATTEMPTS: u32`, `DEFAULT_REQS_PER_SEC: u32`, `DEFAULT_WALL_TIME: Duration`, `DEFAULT_MAX_RESPONSE_BYTES: u64`, `default_user_agent() -> String`

- [ ] **Step 1: Update the toolchain**

```bash
rustup update stable
rustc --version   # must print 1.98.0 or later
```

- [ ] **Step 2: Create `Cargo.toml`**

```toml
[package]
name = "ensemblrest"
version = "0.1.0"
edition = "2024"
rust-version = "1.98"
authors = ["Steve Moss <gawbul@gmail.com>"]
license = "MIT"
description = "A Rust client library for the Ensembl REST API"
repository = "https://github.com/gawbul/ensemblrest-rs"
documentation = "https://docs.rs/ensemblrest"
readme = "README.md"
keywords = ["ensembl", "bioinformatics", "genomics", "rest", "api"]
categories = ["api-bindings", "science"]

[dependencies]
ureq = "3.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]

[lints.rust]
missing_docs = "warn"

[lints.clippy]
all = { level = "warn", priority = -1 }
```

- [ ] **Step 3: Create `.gitignore`**

```
/target
Cargo.lock
*.profraw
/coverage
```

Note: `Cargo.lock` is ignored because this is a library, matching cargo convention.

- [ ] **Step 4: Create `LICENSE`**

Copy verbatim from `/Users/gawbul/Documents/Code/goensemblrest/LICENSE`:

```bash
cp /Users/gawbul/Documents/Code/goensemblrest/LICENSE LICENSE
grep -i copyright LICENSE   # must read: Copyright (c) 2020-2026 Steve Moss
```

- [ ] **Step 5: Write the failing test in `src/lib.rs`**

```rust
//! A Rust client library for the [Ensembl REST API](https://rest.ensembl.org/).
//!
//! This crate is a port of [`goensemblrest`](https://github.com/gawbul/goensemblrest),
//! which is itself a port of [`pyEnsemblRest`](https://github.com/gawbul/pyEnsemblRest).

use std::time::Duration;

/// The current version of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The official Ensembl REST API endpoint.
pub const DEFAULT_BASE_URL: &str = "https://rest.ensembl.org";

/// The default MIME type used for requests and responses.
pub const DEFAULT_CONTENT_TYPE: &str = "application/json";

/// The default per-request timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// The default maximum number of attempts for a single call.
pub const DEFAULT_MAX_ATTEMPTS: u32 = 5;

/// The default number of requests permitted per rate-limit window.
pub const DEFAULT_REQS_PER_SEC: u32 = 15;

/// The default rate-limit sliding-window duration.
pub const DEFAULT_WALL_TIME: Duration = Duration::from_secs(1);

/// The default cap on a single response body, in bytes (100 MiB).
///
/// `ureq` defaults to 10 MiB, which several Ensembl endpoints exceed.
pub const DEFAULT_MAX_RESPONSE_BYTES: u64 = 100 * 1024 * 1024;

/// Returns the default `User-Agent` header value.
pub fn default_user_agent() -> String {
    format!("ensemblrest/{VERSION} (Rust 1.98; +https://github.com/gawbul/ensemblrest-rs)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_go_port() {
        assert_eq!(DEFAULT_BASE_URL, "https://rest.ensembl.org");
        assert_eq!(DEFAULT_CONTENT_TYPE, "application/json");
        assert_eq!(DEFAULT_TIMEOUT, Duration::from_secs(60));
        assert_eq!(DEFAULT_MAX_ATTEMPTS, 5);
        assert_eq!(DEFAULT_REQS_PER_SEC, 15);
        assert_eq!(DEFAULT_WALL_TIME, Duration::from_secs(1));
    }

    #[test]
    fn user_agent_names_the_crate_and_version() {
        let ua = default_user_agent();
        assert!(ua.starts_with("ensemblrest/"), "got {ua}");
        assert!(ua.contains(VERSION), "got {ua}");
        assert!(ua.contains("github.com/gawbul/ensemblrest-rs"), "got {ua}");
    }
}
```

- [ ] **Step 6: Run the tests**

Run: `cargo test`
Expected: PASS, 2 tests.

- [ ] **Step 7: Verify the dependency budget**

```bash
cargo tree --depth 1
```
Expected: exactly three direct dependencies — `ureq`, `serde`, `serde_json`.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml .gitignore LICENSE src/lib.rs
git commit -m "feat: scaffold ensemblrest crate with defaults"
```

---

### Task 2: Rate limiting

**Files:**
- Create: `src/ratelimit.rs`
- Modify: `src/lib.rs` (add `pub mod ratelimit;` and re-export)

**Interfaces:**
- Consumes: nothing from other tasks
- Produces: `RateLimitInfo { reset: Option<i64>, limit: Option<i64>, remaining: Option<i64>, period: Option<i64>, retry_after: Option<f64> }` (derives `Debug, Clone, Default, PartialEq, Serialize, Deserialize`), and `pub(crate) struct RateLimiter` with `RateLimiter::new(reqs: u32, window: Duration) -> Self`, `wait(&self)`, `update_from_headers(&self, &ureq::http::HeaderMap) -> RateLimitInfo`, `info(&self) -> RateLimitInfo`

- [ ] **Step 1: Write the failing tests in `src/ratelimit.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ureq::http::{HeaderMap, HeaderValue};

    #[test]
    fn admits_up_to_capacity_without_sleeping() {
        let rl = RateLimiter::new(3, Duration::from_secs(10));
        let start = Instant::now();
        for _ in 0..3 {
            rl.wait();
        }
        assert!(start.elapsed() < Duration::from_millis(50), "should not have slept");
    }

    #[test]
    fn blocks_once_the_window_is_full() {
        let rl = RateLimiter::new(2, Duration::from_millis(300));
        let start = Instant::now();
        for _ in 0..3 {
            rl.wait();
        }
        // The third call must wait for the first timestamp to age out.
        assert!(start.elapsed() >= Duration::from_millis(250), "elapsed {:?}", start.elapsed());
    }

    #[test]
    fn parses_all_telemetry_headers() {
        let rl = RateLimiter::new(15, Duration::from_secs(1));
        let mut h = HeaderMap::new();
        h.insert("X-RateLimit-Reset", HeaderValue::from_static("42"));
        h.insert("X-RateLimit-Limit", HeaderValue::from_static("55000"));
        h.insert("X-RateLimit-Remaining", HeaderValue::from_static("54999"));
        h.insert("X-RateLimit-Period", HeaderValue::from_static("3600"));
        h.insert("Retry-After", HeaderValue::from_static("2.5"));

        let info = rl.update_from_headers(&h);
        assert_eq!(info.reset, Some(42));
        assert_eq!(info.limit, Some(55000));
        assert_eq!(info.remaining, Some(54999));
        assert_eq!(info.period, Some(3600));
        assert_eq!(info.retry_after, Some(2.5));
        assert_eq!(rl.info(), info, "info() must return the stored telemetry");
    }

    #[test]
    fn ignores_absent_and_unparseable_headers() {
        let rl = RateLimiter::new(15, Duration::from_secs(1));
        let mut h = HeaderMap::new();
        h.insert("X-RateLimit-Reset", HeaderValue::from_static("not-a-number"));
        let info = rl.update_from_headers(&h);
        assert_eq!(info, RateLimitInfo::default());
    }

    #[test]
    fn telemetry_persists_across_responses_that_omit_headers() {
        let rl = RateLimiter::new(15, Duration::from_secs(1));
        let mut h = HeaderMap::new();
        h.insert("X-RateLimit-Limit", HeaderValue::from_static("100"));
        rl.update_from_headers(&h);

        let info = rl.update_from_headers(&HeaderMap::new());
        assert_eq!(info.limit, Some(100), "previously seen values must be retained");
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --lib ratelimit`
Expected: FAIL — `cannot find type RateLimiter in this scope`.

- [ ] **Step 3: Write the implementation in `src/ratelimit.rs`**

```rust
//! Client-side sliding-window rate limiting and rate-limit telemetry.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use ureq::http::HeaderMap;

/// Rate-limit telemetry parsed from Ensembl response headers.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Seconds until the current window resets (`X-RateLimit-Reset`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset: Option<i64>,
    /// Maximum requests permitted per period (`X-RateLimit-Limit`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Requests remaining in the current window (`X-RateLimit-Remaining`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<i64>,
    /// Length of the window in seconds (`X-RateLimit-Period`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<i64>,
    /// Seconds to wait before retrying (`Retry-After`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<f64>,
}

#[derive(Debug, Default)]
struct State {
    stamps: VecDeque<Instant>,
    info: RateLimitInfo,
}

/// A thread-safe sliding-window rate limiter.
#[derive(Debug)]
pub(crate) struct RateLimiter {
    state: Mutex<State>,
    reqs: usize,
    window: Duration,
}

impl RateLimiter {
    /// Creates a limiter admitting `reqs` requests per `window`.
    pub(crate) fn new(reqs: u32, window: Duration) -> Self {
        Self {
            state: Mutex::new(State::default()),
            reqs: reqs.max(1) as usize,
            window,
        }
    }

    /// Blocks until another request is permitted under the sliding window.
    pub(crate) fn wait(&self) {
        loop {
            let sleep_for = {
                // A poisoned lock means another thread panicked mid-update; the
                // queue is still structurally valid, so recover rather than
                // propagate a panic into unrelated callers.
                let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
                let now = Instant::now();

                while let Some(&oldest) = st.stamps.front() {
                    if now.duration_since(oldest) >= self.window {
                        st.stamps.pop_front();
                    } else {
                        break;
                    }
                }

                if st.stamps.len() < self.reqs {
                    st.stamps.push_back(now);
                    return;
                }

                let oldest = *st.stamps.front().expect("queue is at capacity");
                self.window.saturating_sub(now.duration_since(oldest))
            };

            if sleep_for.is_zero() {
                continue;
            }
            std::thread::sleep(sleep_for);
        }
    }

    /// Merges rate-limit telemetry from response headers and returns the current state.
    ///
    /// Headers that are absent or unparseable leave the previous value untouched,
    /// so telemetry persists across responses that omit it.
    pub(crate) fn update_from_headers(&self, headers: &HeaderMap) -> RateLimitInfo {
        let get_i64 = |name: &str| -> Option<i64> {
            headers.get(name)?.to_str().ok()?.trim().parse().ok()
        };
        let get_f64 = |name: &str| -> Option<f64> {
            headers.get(name)?.to_str().ok()?.trim().parse().ok()
        };

        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(v) = get_i64("X-RateLimit-Reset") {
            st.info.reset = Some(v);
        }
        if let Some(v) = get_i64("X-RateLimit-Limit") {
            st.info.limit = Some(v);
        }
        if let Some(v) = get_i64("X-RateLimit-Remaining") {
            st.info.remaining = Some(v);
        }
        if let Some(v) = get_i64("X-RateLimit-Period") {
            st.info.period = Some(v);
        }
        if let Some(v) = get_f64("Retry-After") {
            st.info.retry_after = Some(v);
        }
        st.info.clone()
    }

    /// Returns a copy of the current rate-limit telemetry.
    pub(crate) fn info(&self) -> RateLimitInfo {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .info
            .clone()
    }
}
```

- [ ] **Step 4: Wire the module into `src/lib.rs`**

```rust
pub mod ratelimit;

pub use ratelimit::RateLimitInfo;
```

- [ ] **Step 5: Run the tests**

Run: `cargo test --lib ratelimit`
Expected: PASS, 5 tests. The `blocks_once_the_window_is_full` test takes ~300 ms.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
git add src/ratelimit.rs src/lib.rs
git commit -m "feat: add sliding-window rate limiter and header telemetry"
```

---

### Task 3: Errors

**Files:**
- Create: `src/error.rs`
- Modify: `src/lib.rs` (add `pub mod error;` and re-exports)

**Interfaces:**
- Consumes: `RateLimitInfo` from Task 2, via `use crate::ratelimit::RateLimitInfo;`
- Produces: `Error`, `Result<T>`, `ApiError { status: u16, message: String, rate_limit: RateLimitInfo, body: Vec<u8> }`, `ApiErrorKind`, `ApiError::kind() -> ApiErrorKind`, `Error::api_kind() -> Option<ApiErrorKind>`, `Error::is_not_found() -> bool`, `status_description(u16) -> Option<(&'static str, &'static str)>`, `reason_phrase(u16) -> &'static str`, `parse_error_message(&[u8], &str) -> String`

- [ ] **Step 1: Write the failing tests in `src/error.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn api(status: u16, message: &str) -> ApiError {
        ApiError {
            status,
            message: message.to_string(),
            rate_limit: RateLimitInfo::default(),
            body: message.as_bytes().to_vec(),
        }
    }

    #[test]
    fn display_matches_the_go_port_exactly() {
        let e = api(404, "ID not found");
        assert_eq!(
            e.to_string(),
            "EnsEMBL REST API returned a 404 (Not Found): ID not found"
        );
    }

    #[test]
    fn display_appends_retry_after_for_429() {
        let mut e = api(429, "slow down");
        e.rate_limit.retry_after = Some(3.0);
        assert_eq!(
            e.to_string(),
            "EnsEMBL REST API returned a 429 (Too Many Requests): slow down \
             (Rate limit hit: Retry after 3 seconds)"
        );
    }

    #[test]
    fn empty_message_falls_back_to_the_status_description() {
        let e = api(503, "");
        assert_eq!(
            e.to_string(),
            "EnsEMBL REST API returned a 503 (Service Unavailable): \
             The service is temporarily down; retry after a pause"
        );
    }

    #[test]
    fn kind_maps_each_documented_status() {
        assert_eq!(api(400, "").kind(), ApiErrorKind::BadRequest);
        assert_eq!(api(404, "").kind(), ApiErrorKind::NotFound);
        assert_eq!(api(408, "").kind(), ApiErrorKind::Timeout);
        assert_eq!(api(429, "").kind(), ApiErrorKind::RateLimit);
        assert_eq!(api(500, "").kind(), ApiErrorKind::InternalServer);
        assert_eq!(api(503, "").kind(), ApiErrorKind::ServiceUnavailable);
        assert_eq!(api(418, "").kind(), ApiErrorKind::Other(418));
    }

    #[test]
    fn max_retries_exposes_the_underlying_api_kind_through_source() {
        let inner = Error::Api(api(503, "down"));
        let e = Error::MaxRetries { attempts: 5, last: Box::new(inner) };
        assert_eq!(e.api_kind(), Some(ApiErrorKind::ServiceUnavailable));
        assert!(std::error::Error::source(&e).is_some());
    }

    #[test]
    fn is_not_found_is_a_convenience_over_api_kind() {
        assert!(Error::Api(api(404, "")).is_not_found());
        assert!(!Error::Api(api(400, "")).is_not_found());
    }

    #[test]
    fn parse_error_message_prefers_json_error_then_message_then_raw() {
        assert_eq!(parse_error_message(br#"{"error":"boom"}"#, "d"), "boom");
        assert_eq!(parse_error_message(br#"{"message":"bang"}"#, "d"), "bang");
        assert_eq!(parse_error_message(b"not json", "d"), "not json");
        assert_eq!(parse_error_message(b"", "d"), "d");
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --lib error`
Expected: FAIL — `cannot find type ApiError in this scope`.

- [ ] **Step 3: Write the implementation in `src/error.rs`**

```rust
//! Error types for the Ensembl REST API client.

use std::fmt;

use crate::ratelimit::RateLimitInfo;

/// A convenience alias for results returned by this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Any error produced by this crate.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The API returned a non-2xx status.
    Api(ApiError),
    /// All retry attempts were exhausted.
    MaxRetries {
        /// How many attempts were made.
        attempts: u32,
        /// The error from the final attempt.
        last: Box<Error>,
    },
    /// A mandatory path parameter was missing or empty.
    MissingParam(String),
    /// The endpoint name given to [`crate::Client::call`] is not in the table.
    UnknownEndpoint(String),
    /// The underlying HTTP transport failed.
    Transport(Box<ureq::Error>),
    /// The response body could not be deserialized as JSON.
    Decode(serde_json::Error),
    /// A [`crate::ClientBuilder`] value was rejected.
    InvalidConfig(String),
    /// The response body was not valid UTF-8.
    InvalidUtf8(std::string::FromUtf8Error),
}

/// A structured non-2xx response from the Ensembl REST API.
#[derive(Debug, Clone, Default)]
pub struct ApiError {
    /// The HTTP status code.
    pub status: u16,
    /// The error message, extracted from the body where possible.
    pub message: String,
    /// Rate-limit telemetry from the response headers.
    pub rate_limit: RateLimitInfo,
    /// The raw response body.
    pub body: Vec<u8>,
}

/// The category of an [`ApiError`], replacing the Go port's sentinel errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ApiErrorKind {
    /// HTTP 400.
    BadRequest,
    /// HTTP 404.
    NotFound,
    /// HTTP 408.
    Timeout,
    /// HTTP 429.
    RateLimit,
    /// HTTP 500.
    InternalServer,
    /// HTTP 503.
    ServiceUnavailable,
    /// Any other status.
    Other(u16),
}

impl ApiError {
    /// Classifies this error by status code.
    pub fn kind(&self) -> ApiErrorKind {
        match self.status {
            400 => ApiErrorKind::BadRequest,
            404 => ApiErrorKind::NotFound,
            408 => ApiErrorKind::Timeout,
            429 => ApiErrorKind::RateLimit,
            500 => ApiErrorKind::InternalServer,
            503 => ApiErrorKind::ServiceUnavailable,
            other => ApiErrorKind::Other(other),
        }
    }
}

/// Ensembl-specific descriptions for the status codes it documents.
///
/// Returns `(name, description)`. Ported verbatim from the Go port's
/// `HTTPStatusDescriptions`.
pub fn status_description(status: u16) -> Option<(&'static str, &'static str)> {
    Some(match status {
        200 => ("OK", "Request was a success. Only process data from the service when you receive this code"),
        400 => ("Bad Request", "Occurs during exceptional circumstances such as the service is unable to find an ID. If JSON, the object contains the error message"),
        403 => ("Forbidden", "You are submitting far too many requests and have been temporarily forbidden access. Wait and retry with a maximum of 15 requests per second"),
        404 => ("Not Found", "Indicates a badly formatted request. Check your URL"),
        408 => ("Timeout", "The request was not processed in time. Wait and retry later"),
        415 => ("Unsupported Media Type", "The server is refusing to service the request because the entity format is not supported"),
        429 => ("Too Many Requests", "You have been rate-limited; wait and retry"),
        500 => ("Internal Server Error", "Internal server error. Check your input or contact the Ensembl team if issue persists"),
        503 => ("Service Unavailable", "The service is temporarily down; retry after a pause"),
        _ => return None,
    })
}

/// Returns the canonical HTTP reason phrase, the equivalent of Go's `http.StatusText`.
pub fn reason_phrase(status: u16) -> &'static str {
    ureq::http::StatusCode::from_u16(status)
        .ok()
        .and_then(|c| c.canonical_reason())
        .unwrap_or("")
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = status_description(self.status);
        let name = desc.map_or_else(|| reason_phrase(self.status), |d| d.0);
        let msg: &str = if self.message.is_empty() {
            desc.map_or("", |d| d.1)
        } else {
            &self.message
        };

        if self.status == 0 {
            return write!(f, "EnsEMBL REST API error: {msg}");
        }
        if self.status == 429 {
            if let Some(retry_after) = self.rate_limit.retry_after {
                return write!(
                    f,
                    "EnsEMBL REST API returned a {} ({name}): {msg} \
                     (Rate limit hit: Retry after {} seconds)",
                    self.status, retry_after as i64
                );
            }
        }
        write!(f, "EnsEMBL REST API returned a {} ({name}): {msg}", self.status)
    }
}

impl std::error::Error for ApiError {}

impl Error {
    /// Returns the [`ApiErrorKind`] of this error, following `MaxRetries` chains.
    ///
    /// This replaces the Go port's `errors.Is(err, ErrNotFound)` pattern.
    pub fn api_kind(&self) -> Option<ApiErrorKind> {
        match self {
            Error::Api(e) => Some(e.kind()),
            Error::MaxRetries { last, .. } => last.api_kind(),
            _ => None,
        }
    }

    /// Returns `true` if this is an HTTP 400.
    pub fn is_bad_request(&self) -> bool {
        self.api_kind() == Some(ApiErrorKind::BadRequest)
    }

    /// Returns `true` if this is an HTTP 404.
    pub fn is_not_found(&self) -> bool {
        self.api_kind() == Some(ApiErrorKind::NotFound)
    }

    /// Returns `true` if this is an HTTP 429.
    pub fn is_rate_limited(&self) -> bool {
        self.api_kind() == Some(ApiErrorKind::RateLimit)
    }

    /// Returns `true` if this is an HTTP 503.
    pub fn is_service_unavailable(&self) -> bool {
        self.api_kind() == Some(ApiErrorKind::ServiceUnavailable)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Api(e) => write!(f, "{e}"),
            Error::MaxRetries { attempts, last } => {
                write!(f, "ensembl: maximum retry attempts reached: {last} (attempts: {attempts})")
            }
            Error::MissingParam(name) => write!(f, "mandatory param \"{name}\" not specified"),
            Error::UnknownEndpoint(name) => {
                write!(f, "unknown Ensembl REST API endpoint \"{name}\"")
            }
            Error::Transport(e) => write!(f, "ensembl: transport error: {e}"),
            Error::Decode(e) => write!(f, "failed to decode response as JSON: {e}"),
            Error::InvalidConfig(msg) => write!(f, "ensembl: invalid configuration: {msg}"),
            Error::InvalidUtf8(e) => write!(f, "ensembl: response was not valid UTF-8: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Api(e) => Some(e),
            Error::MaxRetries { last, .. } => Some(&**last),
            Error::Transport(e) => Some(&**e),
            Error::Decode(e) => Some(e),
            Error::InvalidUtf8(e) => Some(e),
            _ => None,
        }
    }
}

/// Extracts an error message from a response body.
///
/// Tries the JSON `error` field, then the JSON `message` field, then the raw body,
/// then `default_msg` for an empty body. Ported from the Go port's `parseErrorMessage`.
pub fn parse_error_message(body: &[u8], default_msg: &str) -> String {
    if body.is_empty() {
        return default_msg.to_string();
    }

    #[derive(serde::Deserialize)]
    struct Wire {
        #[serde(default)]
        error: String,
        #[serde(default)]
        message: String,
    }

    if let Ok(w) = serde_json::from_slice::<Wire>(body) {
        if !w.error.is_empty() {
            return w.error;
        }
        if !w.message.is_empty() {
            return w.message;
        }
    }
    String::from_utf8_lossy(body).into_owned()
}
```

- [ ] **Step 4: Wire the module into `src/lib.rs`**

Add below the consts:

```rust
pub mod error;

pub use error::{ApiError, ApiErrorKind, Error, Result};
```

- [ ] **Step 5: Run the tests**

Run: `cargo test --lib error`
Expected: PASS, 7 tests.

- [ ] **Step 6: Cross-check the Display strings against the Go port**

```bash
grep -n 'EnsEMBL REST API returned' /Users/gawbul/Documents/Code/goensemblrest/errors.go
```
Confirm the format strings match character for character, including the `(Rate limit hit: Retry after %d seconds)` suffix.

- [ ] **Step 7: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
git add src/error.rs src/lib.rs
git commit -m "feat: add error types with Go-parity Display strings"
```

---

### Task 4: Encoding and path resolution

**Files:**
- Create: `src/encoding.rs`
- Modify: `src/lib.rs` (add `pub(crate) mod encoding;`)

**Interfaces:**
- Consumes: `Error`, `Result` from Task 3
- Produces: `pub(crate) fn encode_path_segment(&str) -> String`, `pub(crate) fn encode_form_component(&str) -> String`, `pub(crate) fn resolve_path(template: &str, params: &[(&str, &str)]) -> Result<String>`, `pub(crate) fn encode_query(pairs: &[(&str, &str)]) -> String`

This is the highest-risk pure logic in the crate. The colon rule is not a nicety: every
genomic-region endpoint breaks without it.

- [ ] **Step 1: Write the failing tests in `src/encoding.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_encoding_preserves_colons_for_genomic_regions() {
        // The single most important assertion in this crate.
        assert_eq!(
            encode_path_segment("13:32889611..32973805:1"),
            "13:32889611..32973805:1"
        );
        assert_eq!(encode_path_segment("homo_sapiens:BRCA2"), "homo_sapiens:BRCA2");
        assert_eq!(encode_path_segment("X:1000..2000:1"), "X:1000..2000:1");
    }

    #[test]
    fn path_encoding_keeps_the_unreserved_set() {
        assert_eq!(encode_path_segment("abcXYZ019-._~"), "abcXYZ019-._~");
    }

    #[test]
    fn path_encoding_escapes_everything_else() {
        assert_eq!(encode_path_segment("a/b"), "a%2Fb");
        assert_eq!(encode_path_segment("a b"), "a%20b");
        assert_eq!(encode_path_segment("a?b"), "a%3Fb");
        assert_eq!(encode_path_segment("a#b"), "a%23b");
        assert_eq!(encode_path_segment("a%b"), "a%25b");
        assert_eq!(encode_path_segment("a&b"), "a%26b");
    }

    #[test]
    fn path_encoding_handles_multibyte_utf8() {
        // Each UTF-8 byte is escaped individually, uppercase hex.
        assert_eq!(encode_path_segment("é"), "%C3%A9");
    }

    #[test]
    fn resolve_path_substitutes_named_parameters() {
        assert_eq!(
            resolve_path("/lookup/id/{{id}}", &[("id", "ENSG00000157764")]).unwrap(),
            "/lookup/id/ENSG00000157764"
        );
    }

    #[test]
    fn resolve_path_substitutes_multiple_parameters() {
        assert_eq!(
            resolve_path(
                "/sequence/region/{{species}}/{{region}}",
                &[("species", "homo_sapiens"), ("region", "X:1000..2000:1")]
            )
            .unwrap(),
            "/sequence/region/homo_sapiens/X:1000..2000:1"
        );
    }

    #[test]
    fn resolve_path_leaves_templates_without_placeholders_alone() {
        assert_eq!(resolve_path("/info/ping", &[]).unwrap(), "/info/ping");
    }

    #[test]
    fn resolve_path_rejects_missing_and_empty_parameters() {
        let err = resolve_path("/lookup/id/{{id}}", &[]).unwrap_err();
        assert!(matches!(&err, Error::MissingParam(n) if n == "id"), "got {err:?}");

        let err = resolve_path("/lookup/id/{{id}}", &[("id", "")]).unwrap_err();
        assert!(matches!(&err, Error::MissingParam(n) if n == "id"), "got {err:?}");
    }

    #[test]
    fn query_encoding_sorts_keys_like_go_url_values() {
        // Go's url.Values.Encode() sorts by key.
        assert_eq!(
            encode_query(&[("zebra", "1"), ("alpha", "2"), ("mid", "3")]),
            "alpha=2&mid=3&zebra=1"
        );
    }

    #[test]
    fn query_encoding_preserves_insertion_order_within_a_key() {
        assert_eq!(
            encode_query(&[("feature", "gene"), ("feature", "transcript")]),
            "feature=gene&feature=transcript"
        );
    }

    #[test]
    fn query_encoding_uses_form_escaping_not_path_escaping() {
        // Space becomes '+', and ':' IS escaped here, unlike in paths.
        assert_eq!(encode_query(&[("q", "a b")]), "q=a+b");
        assert_eq!(encode_query(&[("q", "a:b")]), "q=a%3Ab");
    }

    #[test]
    fn query_encoding_of_nothing_is_empty() {
        assert_eq!(encode_query(&[]), "");
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --lib encoding`
Expected: FAIL — `cannot find function encode_path_segment in this scope`.

- [ ] **Step 3: Write the implementation in `src/encoding.rs`**

```rust
//! Percent-encoding, path-template resolution and query-string building.
//!
//! Hand-written on `std` so the crate needs neither `percent-encoding`, `url`
//! nor `regex`.

use crate::error::{Error, Result};

const HEX: &[u8; 16] = b"0123456789ABCDEF";

fn push_escaped(out: &mut String, byte: u8) {
    out.push('%');
    out.push(HEX[(byte >> 4) as usize] as char);
    out.push(HEX[(byte & 0x0F) as usize] as char);
}

/// Returns `true` for bytes that may appear literally in a path segment.
///
/// This is the RFC 3986 unreserved set plus `:`. The colon is deliberate and
/// load-bearing: Ensembl genomic coordinates such as `13:32889611..32973805:1`
/// and species-qualified symbols such as `homo_sapiens:BRCA2` are rejected by
/// the API if the colon arrives percent-encoded. `.` is unreserved, so the `..`
/// range syntax survives without special handling.
const fn is_path_safe(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~' | b':')
}

/// Returns `true` for bytes that may appear literally in a form-encoded component.
///
/// This is the unreserved set only. Note `:` is *not* included: query strings
/// follow `application/x-www-form-urlencoded`, matching Go's `url.QueryEscape`.
const fn is_form_safe(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~')
}

/// Percent-encodes a URL path segment, preserving colons.
pub(crate) fn encode_path_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        if is_path_safe(b) {
            out.push(b as char);
        } else {
            push_escaped(&mut out, b);
        }
    }
    out
}

/// Percent-encodes a query-string key or value, encoding space as `+`.
pub(crate) fn encode_form_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b if is_form_safe(b) => out.push(b as char),
            b' ' => out.push('+'),
            b => push_escaped(&mut out, b),
        }
    }
    out
}

/// Substitutes `{{name}}` placeholders in a path template with encoded values.
///
/// Returns [`Error::MissingParam`] if a placeholder has no corresponding entry
/// in `params`, or if its value is empty — matching the Go port, which treats an
/// empty value as absent.
pub(crate) fn resolve_path(template: &str, params: &[(&str, &str)]) -> Result<String> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        let Some(rel_end) = rest[start + 2..].find("}}") else {
            // An unclosed placeholder is not a template; emit the remainder as-is.
            break;
        };
        let name = &rest[start + 2..start + 2 + rel_end];

        let value = params
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| *v)
            .unwrap_or("");
        if value.is_empty() {
            return Err(Error::MissingParam(name.to_string()));
        }

        out.push_str(&rest[..start]);
        out.push_str(&encode_path_segment(value));
        rest = &rest[start + 2 + rel_end + 2..];
    }

    out.push_str(rest);
    Ok(out)
}

/// Builds a query string, sorting by key while preserving per-key value order.
///
/// Byte-for-byte compatible with Go's `url.Values.Encode()`, which keeps the
/// URLs produced by this crate and by `goensemblrest` identical.
pub(crate) fn encode_query(pairs: &[(&str, &str)]) -> String {
    if pairs.is_empty() {
        return String::new();
    }

    let mut sorted: Vec<&(&str, &str)> = pairs.iter().collect();
    // A stable sort keeps repeated keys in insertion order, matching url.Values.
    sorted.sort_by(|a, b| a.0.cmp(b.0));

    let mut out = String::new();
    for (i, (k, v)) in sorted.iter().enumerate() {
        if i > 0 {
            out.push('&');
        }
        out.push_str(&encode_form_component(k));
        out.push('=');
        out.push_str(&encode_form_component(v));
    }
    out
}
```

- [ ] **Step 4: Wire the module into `src/lib.rs`**

```rust
pub(crate) mod encoding;
```

- [ ] **Step 5: Run the tests**

Run: `cargo test --lib encoding`
Expected: PASS, 12 tests.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
git add src/encoding.rs src/lib.rs
git commit -m "feat: add path and query encoding with colon preservation"
```

---

### Task 5: Response

**Files:**
- Create: `src/response.rs`
- Modify: `src/lib.rs` (add `pub mod response;` and re-export)

**Interfaces:**
- Consumes: `Error`, `Result` from Task 3; `RateLimitInfo` from Task 2
- Produces: `Response` with `pub(crate) fn new(status: u16, content_type: Option<String>, rate_limit: RateLimitInfo, body: Vec<u8>) -> Self`, `status() -> u16`, `content_type() -> Option<&str>`, `rate_limit() -> &RateLimitInfo`, `bytes() -> &[u8]`, `into_bytes() -> Vec<u8>`, `text() -> Result<String>`, `json<T: DeserializeOwned>() -> Result<T>`

- [ ] **Step 1: Write the failing tests in `src/response.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    fn resp(body: &[u8]) -> Response {
        Response::new(
            200,
            Some("application/json".to_string()),
            RateLimitInfo::default(),
            body.to_vec(),
        )
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Ping {
        ping: i64,
    }

    #[test]
    fn json_deserializes_into_a_struct() {
        assert_eq!(resp(br#"{"ping":1}"#).json::<Ping>().unwrap(), Ping { ping: 1 });
    }

    #[test]
    fn json_deserializes_into_a_value_without_a_model() {
        let v: serde_json::Value = resp(br#"{"a":[1,2]}"#).json().unwrap();
        assert_eq!(v["a"][1], 2);
    }

    #[test]
    fn json_on_malformed_body_is_a_decode_error() {
        let err = resp(b"not json").json::<Ping>().unwrap_err();
        assert!(matches!(err, Error::Decode(_)), "got {err:?}");
    }

    #[test]
    fn text_returns_non_json_payloads() {
        // The FASTA path: this is why Response exists rather than a generic return.
        let fasta = b">ENSG00000157764\nACGT\n";
        assert_eq!(resp(fasta).text().unwrap(), ">ENSG00000157764\nACGT\n");
    }

    #[test]
    fn text_on_invalid_utf8_is_an_error_not_a_panic() {
        let err = resp(&[0xFF, 0xFE]).text().unwrap_err();
        assert!(matches!(err, Error::InvalidUtf8(_)), "got {err:?}");
    }

    #[test]
    fn bytes_and_into_bytes_expose_the_raw_body() {
        let r = resp(b"\x00\x01\x02");
        assert_eq!(r.bytes(), &[0, 1, 2]);
        assert_eq!(r.into_bytes(), vec![0, 1, 2]);
    }

    #[test]
    fn metadata_accessors_report_what_was_constructed() {
        let mut rate = RateLimitInfo::default();
        rate.remaining = Some(7);
        let r = Response::new(404, Some("text/plain".into()), rate, Vec::new());
        assert_eq!(r.status(), 404);
        assert_eq!(r.content_type(), Some("text/plain"));
        assert_eq!(r.rate_limit().remaining, Some(7));
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --lib response`
Expected: FAIL — `cannot find type Response in this scope`.

- [ ] **Step 3: Write the implementation in `src/response.rs`**

```rust
//! The response handle returned by every endpoint method.

use serde::de::DeserializeOwned;

use crate::error::{Error, Result};
use crate::ratelimit::RateLimitInfo;

/// A completed HTTP response from the Ensembl REST API.
///
/// Endpoint methods return this rather than a deserialized value, so the same
/// call site works for JSON and for the text formats Ensembl serves
/// (`text/x-fasta`, `text/x-gff3`, `text/x-phyloxml`, `text/x-nh`).
///
/// # Examples
///
/// ```no_run
/// # use ensemblrest::{Client, types::LookupRecord};
/// # fn main() -> ensemblrest::Result<()> {
/// let client = Client::new()?;
/// let record: LookupRecord = client.get_lookup_by_id("ENSG00000157764", &[])?.json()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Response {
    status: u16,
    content_type: Option<String>,
    rate_limit: RateLimitInfo,
    body: Vec<u8>,
}

impl Response {
    pub(crate) fn new(
        status: u16,
        content_type: Option<String>,
        rate_limit: RateLimitInfo,
        body: Vec<u8>,
    ) -> Self {
        Self { status, content_type, rate_limit, body }
    }

    /// The HTTP status code. Always in the 2xx range for a successful call.
    pub fn status(&self) -> u16 {
        self.status
    }

    /// The `Content-Type` header, if the server sent one.
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    /// Rate-limit telemetry from this response's headers.
    pub fn rate_limit(&self) -> &RateLimitInfo {
        &self.rate_limit
    }

    /// The raw response body.
    pub fn bytes(&self) -> &[u8] {
        &self.body
    }

    /// Consumes the response, returning the raw body.
    pub fn into_bytes(self) -> Vec<u8> {
        self.body
    }

    /// Decodes the body as UTF-8 text.
    ///
    /// Use this for `text/x-fasta`, `text/x-gff3` and the other non-JSON formats.
    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.body.clone()).map_err(Error::InvalidUtf8)
    }

    /// Deserializes the body as JSON.
    ///
    /// `T` may be one of the models in [`crate::types`], your own type, or
    /// [`serde_json::Value`] when you have no model.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.body).map_err(Error::Decode)
    }
}
```

- [ ] **Step 4: Wire the module into `src/lib.rs`**

```rust
pub mod response;

pub use response::Response;
```

- [ ] **Step 5: Run the tests**

Run: `cargo test --lib response`
Expected: PASS, 7 tests. The doctest is `no_run`, so it compiles without network access.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
git add src/response.rs src/lib.rs
git commit -m "feat: add Response with json, text and bytes decoding"
```

---

### Task 6: Request options

**Files:**
- Create: `src/options.rs`
- Modify: `src/lib.rs` (add `pub mod options;` and re-exports)

**Interfaces:**
- Consumes: nothing from other tasks
- Produces: `RequestOption<'a>` (enum with `Query`, `ContentType`, `Header`), free functions `query`, `content_type`, `header`, and `pub(crate) struct RequestConfig<'a> { query: Vec<(&'a str, &'a str)>, headers: Vec<(&'a str, &'a str)>, content_type: &'a str }` with `pub(crate) fn resolve<'a>(default_content_type: &'a str, opts: &'a [RequestOption<'a>]) -> RequestConfig<'a>`

- [ ] **Step 1: Write the failing tests in `src/options.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_options_yields_the_endpoint_default_content_type() {
        let cfg = resolve("application/json", &[]);
        assert_eq!(cfg.content_type, "application/json");
        assert!(cfg.query.is_empty());
        assert!(cfg.headers.is_empty());
    }

    #[test]
    fn repeated_query_options_accumulate_in_order() {
        // This is what replaces the Go port's WithQuery, WithQueryParams and
        // WithURLValues: repeated query() appends exactly like url.Values.Add.
        let opts = [query("feature", "gene"), query("feature", "transcript")];
        let cfg = resolve("application/json", &opts);
        assert_eq!(cfg.query, vec![("feature", "gene"), ("feature", "transcript")]);
    }

    #[test]
    fn content_type_option_overrides_the_endpoint_default() {
        let opts = [content_type("text/x-fasta")];
        let cfg = resolve("application/json", &opts);
        assert_eq!(cfg.content_type, "text/x-fasta");
    }

    #[test]
    fn last_content_type_option_wins() {
        let opts = [content_type("text/x-gff3"), content_type("text/x-fasta")];
        assert_eq!(resolve("application/json", &opts).content_type, "text/x-fasta");
    }

    #[test]
    fn headers_accumulate() {
        let opts = [header("X-One", "1"), header("X-Two", "2")];
        let cfg = resolve("application/json", &opts);
        assert_eq!(cfg.headers, vec![("X-One", "1"), ("X-Two", "2")]);
    }

    #[test]
    fn mixed_options_are_routed_to_the_right_buckets() {
        let opts = [
            query("expand", "1"),
            content_type("text/x-fasta"),
            header("X-Trace", "abc"),
        ];
        let cfg = resolve("application/json", &opts);
        assert_eq!(cfg.query, vec![("expand", "1")]);
        assert_eq!(cfg.headers, vec![("X-Trace", "abc")]);
        assert_eq!(cfg.content_type, "text/x-fasta");
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --lib options`
Expected: FAIL — `cannot find function resolve in this scope`.

- [ ] **Step 3: Write the implementation in `src/options.rs`**

```rust
//! Per-call request customization.

/// A single per-request customization, passed to endpoint methods as a slice.
///
/// Construct these with [`query`], [`content_type`] and [`header`] rather than
/// naming the variants directly.
///
/// # Examples
///
/// ```
/// use ensemblrest::options::{content_type, query};
///
/// let opts = [query("expand", "1"), content_type("text/x-fasta")];
/// assert_eq!(opts.len(), 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RequestOption<'a> {
    /// Appends a query-string parameter. Repeating a key appends another value.
    Query(&'a str, &'a str),
    /// Overrides the `Content-Type` and `Accept` headers for this call.
    ContentType(&'a str),
    /// Sets an additional request header.
    Header(&'a str, &'a str),
}

/// Appends a query-string parameter.
///
/// Repeating the same key appends an additional value, matching Go's
/// `url.Values.Add`.
pub fn query<'a>(key: &'a str, value: &'a str) -> RequestOption<'a> {
    RequestOption::Query(key, value)
}

/// Overrides the `Content-Type` and `Accept` headers for a single call.
///
/// Use this to request the non-JSON formats, for example `"text/x-fasta"`.
pub fn content_type(value: &str) -> RequestOption<'_> {
    RequestOption::ContentType(value)
}

/// Sets an additional request header for a single call.
pub fn header<'a>(key: &'a str, value: &'a str) -> RequestOption<'a> {
    RequestOption::Header(key, value)
}

/// The resolved effect of a slice of [`RequestOption`]s.
#[derive(Debug, Clone)]
pub(crate) struct RequestConfig<'a> {
    pub(crate) query: Vec<(&'a str, &'a str)>,
    pub(crate) headers: Vec<(&'a str, &'a str)>,
    pub(crate) content_type: &'a str,
}

/// Folds `opts` over the endpoint's default content type.
///
/// The effective content type is the last [`RequestOption::ContentType`] given,
/// or `default_content_type` when none is.
pub(crate) fn resolve<'a>(
    default_content_type: &'a str,
    opts: &'a [RequestOption<'a>],
) -> RequestConfig<'a> {
    let mut cfg = RequestConfig {
        query: Vec::new(),
        headers: Vec::new(),
        content_type: default_content_type,
    };
    for opt in opts {
        match *opt {
            RequestOption::Query(k, v) => cfg.query.push((k, v)),
            RequestOption::Header(k, v) => cfg.headers.push((k, v)),
            RequestOption::ContentType(ct) => cfg.content_type = ct,
        }
    }
    cfg
}
```

- [ ] **Step 4: Wire the module into `src/lib.rs`**

```rust
pub mod options;

pub use options::{RequestOption, content_type, header, query};
```

- [ ] **Step 5: Run the tests**

Run: `cargo test --lib options`
Expected: PASS, 6 tests plus 1 doctest.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
git add src/options.rs src/lib.rs
git commit -m "feat: add per-request options"
```

---

### Task 7: Client and builder

**Files:**
- Create: `src/client.rs`
- Modify: `src/lib.rs` (add `pub mod client;` and re-exports)

**Interfaces:**
- Consumes: all `DEFAULT_*` consts and `default_user_agent()` from Task 1; `RateLimiter` from Task 2; `Error`, `Result` from Task 3
- Produces: `Client` (`Clone`), `Client::new() -> Result<Client>`, `Client::builder() -> ClientBuilder`, accessors `base_url() -> &str`, `user_agent() -> &str`, `max_attempts() -> u32`, `rate_limit() -> RateLimitInfo`; `ClientBuilder` with `base_url`, `timeout`, `rate_limit(reqs, window)`, `max_attempts`, `user_agent`, `header`, `max_response_bytes`, `agent`, `build() -> Result<Client>`; and `pub(crate) struct Inner` with fields `agent`, `base_url`, `user_agent`, `headers`, `max_attempts`, `wall_time`, `max_response_bytes`, `limiter`

- [ ] **Step 1: Write the failing tests in `src/client.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_client_uses_the_documented_defaults() {
        let c = Client::new().unwrap();
        assert_eq!(c.base_url(), crate::DEFAULT_BASE_URL);
        assert_eq!(c.max_attempts(), crate::DEFAULT_MAX_ATTEMPTS);
        assert!(c.user_agent().starts_with("ensemblrest/"));
    }

    #[test]
    fn base_url_has_trailing_slashes_trimmed() {
        let c = Client::builder().base_url("https://example.org///").build().unwrap();
        assert_eq!(c.base_url(), "https://example.org");
    }

    #[test]
    fn builder_rejects_zero_timeout() {
        let err = Client::builder().timeout(Duration::ZERO).build().unwrap_err();
        assert!(matches!(err, Error::InvalidConfig(_)), "got {err:?}");
    }

    #[test]
    fn builder_rejects_zero_max_attempts() {
        let err = Client::builder().max_attempts(0).build().unwrap_err();
        assert!(matches!(err, Error::InvalidConfig(_)), "got {err:?}");
    }

    #[test]
    fn builder_rejects_zero_rate_limit_values() {
        let err = Client::builder()
            .rate_limit(0, Duration::from_secs(1))
            .build()
            .unwrap_err();
        assert!(matches!(err, Error::InvalidConfig(_)), "got {err:?}");

        let err = Client::builder()
            .rate_limit(15, Duration::ZERO)
            .build()
            .unwrap_err();
        assert!(matches!(err, Error::InvalidConfig(_)), "got {err:?}");
    }

    #[test]
    fn builder_rejects_an_empty_base_url() {
        let err = Client::builder().base_url("").build().unwrap_err();
        assert!(matches!(err, Error::InvalidConfig(_)), "got {err:?}");
    }

    #[test]
    fn custom_values_are_retained() {
        let c = Client::builder()
            .base_url("https://example.org")
            .max_attempts(2)
            .user_agent("custom/1.0")
            .header("X-Custom", "value")
            .build()
            .unwrap();
        assert_eq!(c.base_url(), "https://example.org");
        assert_eq!(c.max_attempts(), 2);
        assert_eq!(c.user_agent(), "custom/1.0");
        assert_eq!(c.inner.headers, vec![("X-Custom".to_string(), "value".to_string())]);
    }

    #[test]
    fn clones_share_one_rate_limiter() {
        let c = Client::new().unwrap();
        let c2 = c.clone();
        assert!(
            std::sync::Arc::ptr_eq(&c.inner, &c2.inner),
            "clone must share Inner so the limiter and connection pool are shared"
        );
    }

    #[test]
    fn client_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Client>();
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --lib client`
Expected: FAIL — `cannot find type Client in this scope`.

- [ ] **Step 3: Write the implementation in `src/client.rs`**

```rust
//! The Ensembl REST API client and its builder.

use std::sync::Arc;
use std::time::Duration;

use crate::error::{Error, Result};
use crate::ratelimit::{RateLimitInfo, RateLimiter};
use crate::{
    DEFAULT_BASE_URL, DEFAULT_MAX_ATTEMPTS, DEFAULT_MAX_RESPONSE_BYTES, DEFAULT_REQS_PER_SEC,
    DEFAULT_TIMEOUT, DEFAULT_WALL_TIME, default_user_agent,
};

#[derive(Debug)]
pub(crate) struct Inner {
    pub(crate) agent: ureq::Agent,
    pub(crate) base_url: String,
    pub(crate) user_agent: String,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) max_attempts: u32,
    pub(crate) wall_time: Duration,
    pub(crate) max_response_bytes: u64,
    pub(crate) limiter: RateLimiter,
}

/// A client for the Ensembl REST API.
///
/// Cloning is cheap and shares one rate limiter and one connection pool, so a
/// single `Client` cloned across threads observes one global rate limit — the
/// same semantics as sharing a `*Client` between goroutines in the Go port.
///
/// # Examples
///
/// ```no_run
/// # use ensemblrest::Client;
/// # fn main() -> ensemblrest::Result<()> {
/// let client = Client::new()?;
/// let species: serde_json::Value = client.get_info_species(&[])?.json()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) inner: Arc<Inner>,
}

impl Client {
    /// Creates a client with the default configuration.
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    /// Starts configuring a client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// The configured base URL, without a trailing slash.
    pub fn base_url(&self) -> &str {
        &self.inner.base_url
    }

    /// The configured `User-Agent` header value.
    pub fn user_agent(&self) -> &str {
        &self.inner.user_agent
    }

    /// The configured maximum number of attempts per call.
    pub fn max_attempts(&self) -> u32 {
        self.inner.max_attempts
    }

    /// The most recent rate-limit telemetry seen from the API.
    pub fn rate_limit(&self) -> RateLimitInfo {
        self.inner.limiter.info()
    }
}

/// Builder for [`Client`].
#[derive(Debug)]
pub struct ClientBuilder {
    base_url: String,
    timeout: Duration,
    reqs_per_sec: u32,
    wall_time: Duration,
    max_attempts: u32,
    user_agent: String,
    headers: Vec<(String, String)>,
    max_response_bytes: u64,
    agent: Option<ureq::Agent>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: DEFAULT_TIMEOUT,
            reqs_per_sec: DEFAULT_REQS_PER_SEC,
            wall_time: DEFAULT_WALL_TIME,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            user_agent: default_user_agent(),
            headers: Vec::new(),
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            agent: None,
        }
    }
}

impl ClientBuilder {
    /// Sets the API base URL. Trailing slashes are trimmed.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Sets the global per-request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the client-side sliding-window rate limit.
    pub fn rate_limit(mut self, reqs: u32, window: Duration) -> Self {
        self.reqs_per_sec = reqs;
        self.wall_time = window;
        self
    }

    /// Sets the maximum number of attempts per call, including the first.
    pub fn max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Sets the `User-Agent` header value.
    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = ua.into();
        self
    }

    /// Adds a header sent with every request.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Sets the maximum size of a single response body, in bytes.
    ///
    /// Defaults to [`crate::DEFAULT_MAX_RESPONSE_BYTES`] (100 MiB). `ureq`'s own
    /// default is 10 MiB, which large `overlap`, `sequence` and `alignment`
    /// responses exceed.
    pub fn max_response_bytes(mut self, bytes: u64) -> Self {
        self.max_response_bytes = bytes;
        self
    }

    /// Supplies a pre-configured `ureq` agent, bypassing the timeout setting.
    ///
    /// The agent must be built with `http_status_as_error(false)`, otherwise
    /// non-2xx responses surface as transport errors and the retry
    /// classification in this crate cannot see their status codes.
    pub fn agent(mut self, agent: ureq::Agent) -> Self {
        self.agent = Some(agent);
        self
    }

    /// Validates the configuration and builds the client.
    pub fn build(self) -> Result<Client> {
        let base_url = self.base_url.trim_end_matches('/').to_string();
        if base_url.is_empty() {
            return Err(Error::InvalidConfig("base URL must not be empty".into()));
        }
        if self.timeout.is_zero() {
            return Err(Error::InvalidConfig("timeout must be greater than zero".into()));
        }
        if self.reqs_per_sec == 0 {
            return Err(Error::InvalidConfig("reqs_per_sec must be positive".into()));
        }
        if self.wall_time.is_zero() {
            return Err(Error::InvalidConfig("rate limit window must be positive".into()));
        }
        if self.max_attempts < 1 {
            return Err(Error::InvalidConfig("max_attempts must be at least 1".into()));
        }

        let agent = self.agent.unwrap_or_else(|| {
            ureq::Agent::config_builder()
                .timeout_global(Some(self.timeout))
                // Non-2xx must come back as a response, not an error: the retry
                // loop classifies on status code and body content.
                .http_status_as_error(false)
                .build()
                .into()
        });

        Ok(Client {
            inner: Arc::new(Inner {
                agent,
                base_url,
                user_agent: self.user_agent,
                headers: self.headers,
                max_attempts: self.max_attempts,
                wall_time: self.wall_time,
                max_response_bytes: self.max_response_bytes,
                limiter: RateLimiter::new(self.reqs_per_sec, self.wall_time),
            }),
        })
    }
}
```

- [ ] **Step 4: Wire the module into `src/lib.rs`**

```rust
pub mod client;

pub use client::{Client, ClientBuilder};
```

- [ ] **Step 5: Run the tests**

Run: `cargo test --lib client`
Expected: PASS, 9 tests. The `Client` doctest is `no_run` and references
`get_info_species`, which does not exist until Task 13 — until then, comment out that
doctest body or mark it ```ignore``` and restore it in Task 13.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
git add src/client.rs src/lib.rs
git commit -m "feat: add Client and ClientBuilder"
```

---

### Task 8: Mock HTTP server for tests

**Files:**
- Create: `tests/common/mod.rs`, `tests/common/mock.rs`
- Modify: `src/lib.rs` (re-export `serde_json`)

**Interfaces:**
- Consumes: nothing from other tasks
- Produces: `MockServer::start(Vec<MockResponse>) -> MockServer`, `MockServer::base_url() -> &str`, `MockServer::requests() -> Vec<RecordedRequest>`, `MockResponse::json(status: u16, body: &str) -> MockResponse`, `MockResponse::text(status: u16, content_type: &str, body: &str) -> MockResponse`, `MockResponse::with_header(self, k, v) -> Self`, `RecordedRequest { method, target, headers, body }` with `path()`, `query()`, `header(name)`, `json()`

**Why `serde_json` gets re-exported:** integration tests in `tests/` can only use the
crate under test plus `[dev-dependencies]` — a library's regular dependencies are not in
scope. The global constraint forbids dev-dependencies, so `src/lib.rs` re-exports
`serde_json`, and tests reach it as `ensemblrest::serde_json`. This also lets library
users guarantee a matching `serde_json` version, so it is a public-API improvement
rather than a test workaround.

- [ ] **Step 1: Add the re-export to `src/lib.rs`**

```rust
/// Re-export of the `serde_json` version this crate was built against.
///
/// Use this to guarantee your `serde_json::Value` types match the ones
/// [`crate::Response::json`] produces.
pub use serde_json;
```

- [ ] **Step 2: Create `tests/common/mod.rs`**

```rust
//! Shared test helpers.
//!
//! `pub` items here are used by some integration test binaries and not others,
//! which Rust reports as dead code per-binary.
#![allow(dead_code)]

pub mod mock;
```

- [ ] **Step 3: Write `tests/common/mock.rs`**

```rust
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
}

impl MockServer {
    /// Starts a server that answers requests from `responses` in order.
    ///
    /// Once the queue is exhausted every further request gets a 500. Scripting
    /// one response per expected attempt is how the retry tests assert attempt
    /// counts.
    pub fn start(responses: Vec<MockResponse>) -> Self {
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
                if let Ok(req) = serve_one(stream, scripted) {
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
        assert_eq!(reqs.len(), 1, "expected exactly one request, got {}", reqs.len());
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

fn serve_one(mut stream: TcpStream, scripted: Option<MockResponse>) -> std::io::Result<RecordedRequest> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut parts = request_line.trim_end().split_whitespace();
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
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    let response = scripted.unwrap_or_else(|| MockResponse {
        status: 500,
        headers: vec![("Content-Type".into(), "application/json".into())],
        body: br#"{"error":"mock server: no scripted response left"}"#.to_vec(),
    });

    let mut head = format!("HTTP/1.1 {} {}\r\n", response.status, reason(response.status));
    for (k, v) in &response.headers {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    head.push_str(&format!("Content-Length: {}\r\n", response.body.len()));
    // Closing each connection keeps the server free of keep-alive bookkeeping.
    head.push_str("Connection: close\r\n\r\n");

    stream.write_all(head.as_bytes())?;
    stream.write_all(&response.body)?;
    stream.flush()?;

    Ok(RecordedRequest { method, target, headers, body })
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
```

- [ ] **Step 4: Add a self-test at `tests/mock_server.rs`**

The mock server is test infrastructure, so it needs its own test before anything
depends on it.

```rust
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
    assert_eq!(req.header("x-trace"), Some("abc"), "header lookup is case-insensitive");
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
    raw_request(server.base_url(), "GET /a HTTP/1.1\r\nHost: localhost\r\n\r\n");
    let raw = raw_request(server.base_url(), "GET /b HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert!(raw.starts_with("HTTP/1.1 500"), "got {raw}");
    assert_eq!(server.request_count(), 2);
}
```

- [ ] **Step 5: Run the tests**

Run: `cargo test --test mock_server`
Expected: PASS, 3 tests.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
git add src/lib.rs tests/common tests/mock_server.rs
git commit -m "test: add std-only mock HTTP server"
```

---

### Task 9: Request execution and retries

**Files:**
- Create: `src/request.rs`, `tests/retry.rs`
- Modify: `src/lib.rs` (add `pub(crate) mod request;`)

**Interfaces:**
- Consumes: `Client`, `Inner` (Task 7); `resolve_path`, `encode_query` (Task 4); `RequestConfig` (Task 6); `Response` (Task 5); `ApiError`, `Error`, `parse_error_message`, `reason_phrase` (Task 3); `MockServer` (Task 8)
- Produces: `pub(crate) fn is_transient(status: u16, message: &str) -> bool`, and on `Client`: `pub(crate) fn execute(&self, method: Method, path_template: &str, path_params: &[(&str, &str)], body: Option<&serde_json::Value>, cfg: &RequestConfig<'_>) -> Result<Response>`

`Method` is defined in Task 10 but is needed here. Define it in `src/endpoints.rs` as part
of this task with just the enum, and add the rest of that module in Task 10:

```rust
/// The HTTP method an endpoint uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// HTTP GET.
    Get,
    /// HTTP POST.
    Post,
}
```

- [ ] **Step 1: Write the failing unit tests for classification in `src/request.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_side_statuses_are_always_transient() {
        for status in [408, 500, 502, 503, 504] {
            assert!(is_transient(status, ""), "{status} should be transient");
        }
    }

    #[test]
    fn client_side_statuses_are_not_transient() {
        for status in [400, 401, 403, 404, 415] {
            assert!(!is_transient(status, "ordinary message"), "{status} should be fatal");
        }
    }

    #[test]
    fn a_400_is_transient_only_for_known_ensembl_messages() {
        assert!(is_transient(400, "Something bad has happened"));
        assert!(is_transient(
            400,
            "Something went wrong while fetching from LDFeatureContainerAdaptor"
        ));
        assert!(is_transient(400, "Request timeout while processing"));
        assert!(!is_transient(400, "ID 'NOPE' not found"));
    }

    #[test]
    fn transient_message_matching_ignores_case() {
        assert!(is_transient(400, "SOMETHING BAD HAS HAPPENED"));
        assert!(is_transient(400, "something bad has happened"));
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --lib request`
Expected: FAIL — `cannot find function is_transient in this scope`.

- [ ] **Step 3: Write the implementation in `src/request.rs`**

```rust
//! HTTP execution: URL assembly, retries, backoff and response capture.

use std::time::Duration;

use serde_json::Value;

use crate::client::Client;
use crate::encoding::{encode_query, resolve_path};
use crate::endpoints::Method;
use crate::error::{ApiError, Error, Result, parse_error_message, reason_phrase};
use crate::options::RequestConfig;
use crate::response::Response;

/// Body fragments Ensembl returns with HTTP 400 that actually indicate a
/// transient fault. Stored lowercase; comparison lowercases the message.
const TRANSIENT_BODY_MARKERS: [&str; 3] = [
    "something bad has happened",
    "something went wrong while fetching from ldfeaturecontaineradaptor",
    "timeout",
];

/// Returns `true` if a failed response is worth retrying.
///
/// HTTP 429 is handled separately by the caller because it carries `Retry-After`.
pub(crate) fn is_transient(status: u16, message: &str) -> bool {
    if matches!(status, 408 | 500 | 502 | 503 | 504) {
        return true;
    }
    if status == 400 {
        let lower = message.to_lowercase();
        return TRANSIENT_BODY_MARKERS.iter().any(|m| lower.contains(m));
    }
    false
}

/// Applies client headers, per-request headers, then the fixed headers.
///
/// A macro rather than a function because `ureq`'s builder is a different type
/// for requests with and without a body.
macro_rules! apply_headers {
    ($req:expr, $client:expr, $cfg:expr) => {{
        let mut r = $req;
        for (k, v) in &$client.inner.headers {
            r = r.header(k, v);
        }
        for (k, v) in &$cfg.headers {
            r = r.header(*k, *v);
        }
        // Applied last so they win over caller-supplied values, matching Go.
        r = r.header("User-Agent", &$client.inner.user_agent);
        r = r.header("Content-Type", $cfg.content_type);
        r = r.header("Accept", $cfg.content_type);
        r
    }};
}

impl Client {
    /// Performs one HTTP round trip, with no retry or rate-limit logic.
    ///
    /// This is the only method that touches `ureq`, and is therefore the seam
    /// the spec's deferred async backend would replace. Keep transport concerns
    /// in here and policy concerns in [`Client::execute`].
    fn send_once(
        &self,
        method: Method,
        url: &str,
        cfg: &RequestConfig<'_>,
        body: Option<&[u8]>,
    ) -> std::result::Result<ureq::http::Response<ureq::Body>, ureq::Error> {
        match method {
            Method::Get => apply_headers!(self.inner.agent.get(url), self, cfg).call(),
            Method::Post => {
                apply_headers!(self.inner.agent.post(url), self, cfg).send(body.unwrap_or(&[]))
            }
        }
    }

    /// Computes how long to wait before the next attempt.
    fn backoff(&self, attempt: u32, last: Option<&Error>) -> Duration {
        if let Some(Error::Api(api)) = last {
            if let Some(retry_after) = api.rate_limit.retry_after {
                if retry_after > 0.0 {
                    return Duration::from_secs_f64(retry_after);
                }
            }
        }
        (self.inner.wall_time * 2 * attempt).max(Duration::from_millis(10))
    }

    /// Executes a request with rate limiting, retries and backoff.
    pub(crate) fn execute(
        &self,
        method: Method,
        path_template: &str,
        path_params: &[(&str, &str)],
        body: Option<&Value>,
        cfg: &RequestConfig<'_>,
    ) -> Result<Response> {
        let path = resolve_path(path_template, path_params)?;
        let mut url = format!("{}{}", self.inner.base_url, path);
        let query = encode_query(&cfg.query);
        if !query.is_empty() {
            url.push('?');
            url.push_str(&query);
        }

        let body_bytes: Option<Vec<u8>> = match body {
            Some(v) => Some(serde_json::to_vec(v).map_err(Error::Decode)?),
            None => None,
        };

        let mut last: Option<Error> = None;

        for attempt in 1..=self.inner.max_attempts {
            self.inner.limiter.wait();

            match self.send_once(method, &url, cfg, body_bytes.as_deref()) {
                Ok(mut raw) => {
                    let status = raw.status().as_u16();
                    let rate_limit = self.inner.limiter.update_from_headers(raw.headers());
                    let content_type = raw
                        .headers()
                        .get("Content-Type")
                        .and_then(|v| v.to_str().ok())
                        .map(str::to_owned);

                    let bytes = raw
                        .body_mut()
                        .with_config()
                        .limit(self.inner.max_response_bytes)
                        .read_to_vec()
                        .map_err(|e| Error::Transport(Box::new(e)))?;

                    if (200..300).contains(&status) {
                        return Ok(Response::new(status, content_type, rate_limit, bytes));
                    }

                    let message = parse_error_message(&bytes, reason_phrase(status));
                    let api = ApiError { status, message, rate_limit, body: bytes };

                    if !is_transient(api.status, &api.message) && api.status != 429 {
                        return Err(Error::Api(api));
                    }
                    last = Some(Error::Api(api));
                }
                Err(e) => last = Some(Error::Transport(Box::new(e))),
            }

            if attempt < self.inner.max_attempts {
                std::thread::sleep(self.backoff(attempt, last.as_ref()));
            }
        }

        Err(Error::MaxRetries {
            attempts: self.inner.max_attempts,
            // `max_attempts >= 1` is enforced by ClientBuilder::build, so the loop
            // above always ran and always either returned or set `last`.
            last: Box::new(last.expect("at least one attempt was made")),
        })
    }
}
```

- [ ] **Step 4: Wire the module into `src/lib.rs`**

```rust
pub(crate) mod request;
pub mod endpoints;
```

- [ ] **Step 5: Run the unit tests**

Run: `cargo test --lib request`
Expected: PASS, 4 tests.

- [ ] **Step 6: Write the integration tests in `tests/retry.rs`**

These need a temporary way to reach `execute`. Add this to `src/endpoints.rs` now
(Task 10 replaces it with the table-driven version):

```rust
impl crate::Client {
    /// Executes a raw request against an explicit path template.
    ///
    /// Prefer the typed endpoint methods; this exists for tests and for callers
    /// working with paths not yet in [`ENDPOINTS`].
    pub fn call_raw(
        &self,
        method: Method,
        path_template: &str,
        path_params: &[(&str, &str)],
        body: Option<&serde_json::Value>,
        opts: &[crate::RequestOption<'_>],
    ) -> crate::Result<crate::Response> {
        let cfg = crate::options::resolve(crate::DEFAULT_CONTENT_TYPE, opts);
        self.execute(method, path_template, path_params, body, &cfg)
    }
}
```

```rust
mod common;

use common::mock::{MockResponse, MockServer};
use ensemblrest::endpoints::Method;
use ensemblrest::options::{content_type, header, query};
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
    let resp = c.call_raw(Method::Get, "/info/ping", &[], None, &[]).unwrap();

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
        &[content_type("text/x-fasta"), header("X-Call", "per-request")],
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
fn post_sends_a_json_body() {
    let server = MockServer::with_json(200, "[]");
    let c = client(&server, 5);
    let body = serde_json::json!({ "ids": ["ENSG01", "ENSG02"] });
    c.call_raw(Method::Post, "/lookup/id", &[], Some(&body), &[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.json()["ids"][1], "ENSG02");
}

#[test]
fn a_404_is_returned_immediately_without_retrying() {
    let server = MockServer::start(vec![MockResponse::json(404, r#"{"error":"not found"}"#)]);
    let c = client(&server, 5);
    let err = c.call_raw(Method::Get, "/lookup/id/{{id}}", &[("id", "NOPE")], None, &[]).unwrap_err();

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
    let resp = c.call_raw(Method::Get, "/info/ping", &[], None, &[]).unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(server.request_count(), 2);
}

#[test]
fn a_transient_400_is_retried_but_an_ordinary_400_is_not() {
    let transient = MockServer::start(vec![
        MockResponse::json(400, r#"{"error":"something bad has happened"}"#),
        MockResponse::json(200, "{}"),
    ]);
    client(&transient, 5).call_raw(Method::Get, "/a", &[], None, &[]).unwrap();
    assert_eq!(transient.request_count(), 2);

    let fatal = MockServer::start(vec![MockResponse::json(400, r#"{"error":"bad id"}"#)]);
    let err = client(&fatal, 5).call_raw(Method::Get, "/a", &[], None, &[]).unwrap_err();
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
    assert!(start.elapsed() >= Duration::from_millis(50), "Retry-After must be honoured");
}

#[test]
fn exhausting_attempts_yields_max_retries_carrying_the_last_error() {
    let server = MockServer::start(vec![
        MockResponse::json(503, r#"{"error":"down"}"#),
        MockResponse::json(503, r#"{"error":"down"}"#),
    ]);
    let c = client(&server, 2);
    let err = c.call_raw(Method::Get, "/a", &[], None, &[]).unwrap_err();

    assert!(matches!(err, Error::MaxRetries { attempts: 2, .. }), "got {err:?}");
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
    let err = c.call_raw(Method::Get, "/lookup/id/{{id}}", &[], None, &[]).unwrap_err();

    assert!(matches!(&err, Error::MissingParam(n) if n == "id"), "got {err:?}");
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
        .call_raw(Method::Get, "/sequence/id/{{id}}", &[("id", "ENSG01")], None,
                  &[content_type("text/x-fasta")])
        .unwrap();

    assert_eq!(resp.text().unwrap(), ">ENSG00000157764\nACGT\n");
    assert_eq!(resp.content_type(), Some("text/x-fasta"));
}
```

- [ ] **Step 7: Add the test-only backoff shortener to `ClientBuilder`**

`tests/retry.rs` calls `.wall_time_for_test()`. Add it to `src/client.rs`, since the
default 1-second wall time makes the retry tests take tens of seconds:

```rust
impl ClientBuilder {
    /// Shrinks the rate-limit window so retry backoff is fast in tests.
    ///
    /// Backoff is `attempt * wall_time * 2`, so a 5 ms window keeps a
    /// four-retry sequence under 100 ms.
    #[doc(hidden)]
    pub fn wall_time_for_test(self) -> Self {
        self.rate_limit(1000, Duration::from_millis(5))
    }
}
```

- [ ] **Step 8: Run the integration tests**

Run: `cargo test --test retry`
Expected: PASS, 13 tests, completing in under 5 seconds.

- [ ] **Step 9: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
git add src/request.rs src/endpoints.rs src/client.rs src/lib.rs tests/retry.rs
git commit -m "feat: add request execution with retries and backoff"
```

---

### Task 10: Endpoint table and dynamic dispatch

**Files:**
- Modify: `src/endpoints.rs` (add the table, index, `call`, `endpoints()` to the `Method` enum and `call_raw` from Task 9)
- Create: `tests/table.rs`

**Interfaces:**
- Consumes: `Method`, `call_raw`, `execute` (Task 9); `RequestOption`, `resolve` (Task 6); `Error` (Task 3)
- Produces: `EndpointSpec { name: &'static str, doc: &'static str, url: &'static str, method: Method, content_type: &'static str, post_parameters: &'static [&'static str] }`, `pub static ENDPOINTS: &[EndpointSpec]` (106 entries), `pub fn endpoints() -> &'static HashMap<&'static str, &'static EndpointSpec>`, `pub fn endpoint(name: &str) -> Option<&'static EndpointSpec>`, and `Client::call(&self, name: &str, path_params: &[(&str, &str)], body: Option<&serde_json::Value>, opts: &[RequestOption<'_>]) -> Result<Response>`

**Data source:** `/Users/gawbul/Documents/Code/goensemblrest/endpoints.go` is the
authority. This is a mechanical transliteration, not a redesign. Two facts verified
against that file, which simplify the work:

- **All 106 entries use the default content type.** Every `ContentType:` field in the Go
  table is `DefaultContentType`, so every Rust entry sets
  `content_type: crate::DEFAULT_CONTENT_TYPE`.
- **85 are GET, 21 are POST.** Only the 21 POST entries have a non-empty
  `post_parameters`.

- [ ] **Step 1: Extract the Go table for reference**

```bash
cd /Users/gawbul/Documents/Code/goensemblrest
grep -nE '^\t"|Name:|Doc:|URL:|Method:|PostParameters:' endpoints.go > /tmp/go-endpoints.txt
grep -cE '^\t"[a-zA-Z0-9]+": \{' endpoints.go   # must print 106
```

- [ ] **Step 2: Write the failing table tests in `tests/table.rs`**

```rust
use ensemblrest::endpoints::{ENDPOINTS, Method, endpoint, endpoints};

#[test]
fn the_table_has_exactly_the_endpoints_the_go_port_has() {
    assert_eq!(ENDPOINTS.len(), 106, "the Go port's EndpointsTable has 106 entries");
}

#[test]
fn every_name_is_unique() {
    let mut names: Vec<&str> = ENDPOINTS.iter().map(|e| e.name).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(names.len(), before, "duplicate endpoint name in ENDPOINTS");
}

#[test]
fn names_are_camel_case_not_snake_case() {
    // The table keys are the cross-port contract shared with goensemblrest
    // and pyEnsemblRest. Re-casing them silently breaks Client::call for
    // anyone porting code between the three libraries.
    for e in ENDPOINTS {
        assert!(!e.name.contains('_'), "{} must stay camelCase", e.name);
        assert!(
            e.name.starts_with("get") || e.name.starts_with("post"),
            "{} has an unexpected prefix",
            e.name
        );
    }
}

#[test]
fn every_url_is_rooted_and_every_entry_is_documented() {
    for e in ENDPOINTS {
        assert!(e.url.starts_with('/'), "{} url must start with '/': {}", e.name, e.url);
        assert!(!e.doc.is_empty(), "{} must have documentation", e.name);
        assert_eq!(e.content_type, "application/json", "{}", e.name);
    }
}

#[test]
fn only_post_endpoints_declare_post_parameters() {
    for e in ENDPOINTS {
        if e.method == Method::Get {
            assert!(e.post_parameters.is_empty(), "{} is GET but declares body params", e.name);
        }
    }
    let posts = ENDPOINTS.iter().filter(|e| e.method == Method::Post).count();
    assert_eq!(posts, 21, "the Go port has 21 POST endpoints");
}

#[test]
fn placeholders_in_urls_are_well_formed() {
    for e in ENDPOINTS {
        assert_eq!(
            e.url.matches("{{").count(),
            e.url.matches("}}").count(),
            "{} has unbalanced placeholders: {}",
            e.name,
            e.url
        );
    }
}

#[test]
fn lookup_by_name_works_and_is_consistent_with_the_slice() {
    let spec = endpoint("getLookupById").expect("getLookupById must exist");
    assert_eq!(spec.url, "/lookup/id/{{id}}");
    assert_eq!(spec.method, Method::Get);
    assert_eq!(endpoints().len(), ENDPOINTS.len());
    assert!(endpoint("noSuchEndpoint").is_none());
}

#[test]
fn spot_check_against_the_go_table() {
    // One of each shape, transcribed by hand from goensemblrest/endpoints.go.
    let archive_post = endpoint("getArchiveByMultipleIds").unwrap();
    assert_eq!(archive_post.url, "/archive/id");
    assert_eq!(archive_post.method, Method::Post);
    assert_eq!(archive_post.post_parameters, &["id"], "archive uses 'id', not 'ids'");

    let lookup_post = endpoint("getLookupByMultipleIds").unwrap();
    assert_eq!(lookup_post.post_parameters, &["ids"], "lookup uses 'ids', not 'id'");

    let two_params = endpoint("getHomologyBySymbol").unwrap();
    assert_eq!(two_params.url, "/homology/symbol/{{species}}/{{symbol}}");

    let no_params = endpoint("getGA4GHBeacon").unwrap();
    assert_eq!(no_params.url, "/ga4gh/beacon");
    assert_eq!(no_params.method, Method::Get);
}
```

- [ ] **Step 3: Run to verify it fails**

Run: `cargo test --test table`
Expected: FAIL — `cannot find value ENDPOINTS in ensemblrest::endpoints`.

- [ ] **Step 4: Write the module scaffolding in `src/endpoints.rs`**

Keep the `Method` enum and `call_raw` from Task 9 and add:

```rust
//! The catalog of Ensembl REST API endpoints and dynamic dispatch.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde_json::Value;

use crate::error::{Error, Result};
use crate::options::{RequestOption, resolve};
use crate::response::Response;
use crate::{Client, DEFAULT_CONTENT_TYPE};

/// Metadata describing one Ensembl REST API endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndpointSpec {
    /// The camelCase name, shared with `goensemblrest` and `pyEnsemblRest`.
    pub name: &'static str,
    /// A one-line description, taken from the Ensembl documentation.
    pub doc: &'static str,
    /// The path template, with `{{param}}` placeholders.
    pub url: &'static str,
    /// The HTTP method.
    pub method: Method,
    /// The default `Content-Type` and `Accept` value.
    pub content_type: &'static str,
    /// The JSON body keys this endpoint accepts, for POST endpoints.
    pub post_parameters: &'static [&'static str],
}

/// Returns the endpoint catalog, indexed by name.
///
/// Unlike the Go port, which clones its map defensively, this returns a shared
/// reference: the data is immutable `'static`, so there is nothing to defend.
pub fn endpoints() -> &'static HashMap<&'static str, &'static EndpointSpec> {
    static INDEX: OnceLock<HashMap<&'static str, &'static EndpointSpec>> = OnceLock::new();
    INDEX.get_or_init(|| ENDPOINTS.iter().map(|e| (e.name, e)).collect())
}

/// Looks up a single endpoint by name.
pub fn endpoint(name: &str) -> Option<&'static EndpointSpec> {
    endpoints().get(name).copied()
}

impl Client {
    /// Calls any endpoint by its camelCase name.
    ///
    /// This is the equivalent of the Go port's `Call` and of `pyEnsemblRest`'s
    /// dynamic attribute dispatch. Prefer the typed methods where one exists.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnknownEndpoint`] if `name` is not in [`ENDPOINTS`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ensemblrest::Client;
    /// # fn main() -> ensemblrest::Result<()> {
    /// let client = Client::new()?;
    /// let v: serde_json::Value = client
    ///     .call("getLookupById", &[("id", "ENSG00000157764")], None, &[])?
    ///     .json()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn call(
        &self,
        name: &str,
        path_params: &[(&str, &str)],
        body: Option<&Value>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let spec = endpoint(name).ok_or_else(|| Error::UnknownEndpoint(name.to_string()))?;
        let cfg = resolve(spec.content_type, opts);
        self.execute(spec.method, spec.url, path_params, body, &cfg)
    }
}
```

- [ ] **Step 5: Transcribe all 106 entries into `ENDPOINTS`**

Work through `goensemblrest/endpoints.go` top to bottom, preserving its order and its
domain-grouping comments. Apply exactly these rules:

| Go field | Rust field | Transformation |
|---|---|---|
| map key and `Name:` | `name` | Copy verbatim. Never re-case. |
| `Doc:` | `doc` | Copy verbatim. |
| `URL:` | `url` | Copy verbatim, `{{param}}` included. |
| `Method: http.MethodGet` | `method` | `Method::Get` |
| `Method: http.MethodPost` | `method` | `Method::Post` |
| `ContentType: DefaultContentType` | `content_type` | `DEFAULT_CONTENT_TYPE` (all 106) |
| `PostParameters: []string{...}` | `post_parameters` | `&["..."]`, same order |
| field absent | `post_parameters` | `&[]` |

The four shapes, transcribed from the Go source:

```rust
/// The full catalog of Ensembl REST API endpoints.
///
/// Ported entry-for-entry from `goensemblrest`'s `EndpointsTable`, preserving
/// its order and grouping.
pub static ENDPOINTS: &[EndpointSpec] = &[
    // ---- Archive ----

    // GET with one path parameter.
    EndpointSpec {
        name: "getArchiveById",
        doc: "Uses the given identifier to return its latest version",
        url: "/archive/id/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    // POST with a body parameter and no path parameters.
    EndpointSpec {
        name: "getArchiveByMultipleIds",
        doc: "Retrieve the latest version for a set of identifiers",
        url: "/archive/id",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["id"],
    },

    // ---- Comparative Genomics ----

    // GET with two path parameters.
    EndpointSpec {
        name: "getCafeGeneTreeMemberBySymbol",
        doc: "Retrieves the cafe tree of the gene tree that contains the gene identified by a symbol",
        url: "/cafe/genetree/member/symbol/{{species}}/{{symbol}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },

    // ---- GA4GH ----

    // GET with no parameters.
    EndpointSpec {
        name: "getGA4GHBeacon",
        doc: "Return Beacon information",
        url: "/ga4gh/beacon",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },

    // ... continue for all 106 entries, in the Go file's order ...
];
```

- [ ] **Step 6: Verify the transcription mechanically**

```bash
cargo test --test table
```
Expected: PASS, 8 tests. If `the_table_has_exactly_the_endpoints_the_go_port_has` fails,
diff the name lists:

```bash
cd /Users/gawbul/Documents/Code/goensemblrest && \
  grep -oE '^\t"[a-zA-Z0-9]+":' endpoints.go | tr -d '\t":' | sort > /tmp/go-names.txt
\
  grep -oE 'name: "[a-zA-Z0-9]+"' src/endpoints.rs | sed 's/name: "//;s/"//' | sort > /tmp/rs-names.txt
diff /tmp/go-names.txt /tmp/rs-names.txt && echo "IDENTICAL"
```
Expected: `IDENTICAL`.

- [ ] **Step 7: Add dynamic-dispatch tests to `tests/table.rs`**

```rust
use ensemblrest::{Client, Error};

mod common;
use common::mock::MockServer;

#[test]
fn call_dispatches_by_name() {
    let server = MockServer::with_json(200, r#"{"id":"ENSG00000157764"}"#);
    let c = Client::builder().base_url(server.base_url()).build().unwrap();
    let v: serde_json::Value = c
        .call("getLookupById", &[("id", "ENSG00000157764")], None, &[])
        .unwrap()
        .json()
        .unwrap();

    assert_eq!(v["id"], "ENSG00000157764");
    assert_eq!(server.only_request().path(), "/lookup/id/ENSG00000157764");
}

#[test]
fn call_with_an_unknown_name_is_an_error_before_any_request() {
    let server = MockServer::with_json(200, "{}");
    let c = Client::builder().base_url(server.base_url()).build().unwrap();
    let err = c.call("noSuchEndpoint", &[], None, &[]).unwrap_err();

    assert!(matches!(&err, Error::UnknownEndpoint(n) if n == "noSuchEndpoint"), "got {err:?}");
    assert_eq!(server.request_count(), 0);
}
```

Add `mod common;` at the top of `tests/table.rs` for these.

- [ ] **Step 8: Run everything**

Run: `cargo test`
Expected: PASS, all suites.

- [ ] **Step 9: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
git add src/endpoints.rs tests/table.rs
git commit -m "feat: add the 106-endpoint catalog and dynamic dispatch"
```

---

### Task 11: Domain models

**Files:**
- Create: `src/types.rs`
- Modify: `src/lib.rs` (add `pub mod types;`)

**Interfaces:**
- Consumes: nothing from other tasks
- Produces: `PingResponse`, `ArchiveRecord`, `LookupRecord`, `SequenceRecord`, `SpeciesRecord`, `SpeciesResponse`, `AssemblyInfo`, `AssemblyRegionInfo`, `HomologyRecord`, `HomologyResponse`, `XrefRecord`, `VariationRecord`, `LDRecord`, `MappingRecord`, `VEPRecord`, `BeaconResponse`, `BeaconQueryResponse` — all deriving `Debug, Clone, Default, Serialize, Deserialize`

**Data source:** `/Users/gawbul/Documents/Code/goensemblrest/types.go`, from the
`--- Domain Models ---` comment onward. Transcribe with these rules:

| Go | Rust | Note |
|---|---|---|
| `string` | `String` | |
| `int` | `i64` | |
| `*float64` / `*int` / `*bool` | `Option<f64>` / `Option<i64>` / `Option<bool>` | Go pointers model nullability |
| `[]string` | `Vec<String>` | |
| `[]LookupRecord` | `Vec<LookupRecord>` | |
| `*LookupRecord` | `Option<Box<LookupRecord>>` | `Box` breaks the recursive type |
| `json.RawMessage` (object) | `serde_json::Value` | |
| `json.RawMessage` (JSON array) | `Vec<serde_json::Value>` | Indexable without an `as_array` hop |
| anonymous nested struct | a named struct | Name it after its field |
| `json:"foo"` | `#[serde(rename = "foo")]` | Only when snake_case differs from the wire name |

Two rules that are not transcription:

- **`#[serde(default)]` on every container.** Ensembl's response shape varies with query
  parameters, so a missing field must default rather than fail the whole call. Without
  this the models are unusable in practice.
- **`LookupRecord.extra` replaces Go's dead field.** `types.go` declares
  `Extra json.RawMessage \`json:"-"\``, and `json:"-"` means it can never be populated.
  Use `#[serde(flatten)] pub extra: Map<String, Value>` instead, which captures the
  nested data `?expand=1` returns rather than dropping it.

- [ ] **Step 1: Write the failing tests in `src/types.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_deserializes() {
        let p: PingResponse = serde_json::from_str(r#"{"ping":1}"#).unwrap();
        assert_eq!(p.ping, 1);
    }

    #[test]
    fn missing_fields_default_rather_than_failing() {
        // Ensembl omits most fields unless asked for them.
        let r: LookupRecord = serde_json::from_str(r#"{"id":"ENSG00000157764"}"#).unwrap();
        assert_eq!(r.id, "ENSG00000157764");
        assert_eq!(r.species, "");
        assert_eq!(r.start, 0);
        assert!(r.transcripts.is_empty());
    }

    #[test]
    fn wire_names_that_differ_from_field_names_are_renamed() {
        let json = r#"{
            "id":"ENSG00000157764",
            "seq_region_name":"7",
            "object_type":"Gene",
            "display_name":"BRAF",
            "assembly_name":"GRCh38"
        }"#;
        let r: LookupRecord = serde_json::from_str(json).unwrap();
        assert_eq!(r.seq_region_name, "7");
        assert_eq!(r.object_type, "Gene");
        assert_eq!(r.display_name, "BRAF");
        assert_eq!(r.assembly_name, "GRCh38");
    }

    #[test]
    fn expand_extras_are_captured_rather_than_dropped() {
        // The Go port's `Extra` field is tagged `json:"-"` and can never be
        // populated. Flattening captures what ?expand=1 actually returns.
        let json = r#"{"id":"ENSG01","some_expanded_field":{"nested":true}}"#;
        let r: LookupRecord = serde_json::from_str(json).unwrap();
        assert_eq!(r.extra["some_expanded_field"]["nested"], true);
    }

    #[test]
    fn nested_transcripts_round_trip() {
        let json = r#"{"id":"ENSG01","Transcript":[{"id":"ENST01","biotype":"protein_coding"}]}"#;
        let r: LookupRecord = serde_json::from_str(json).unwrap();
        assert_eq!(r.transcripts.len(), 1);
        assert_eq!(r.transcripts[0].id, "ENST01");
        assert_eq!(r.transcripts[0].biotype, "protein_coding");
    }

    #[test]
    fn nullable_numbers_are_options() {
        let with: LDRecord =
            serde_json::from_str(r#"{"variation1":"a","variation2":"b","r2":0.8,"d_prime":null}"#)
                .unwrap();
        assert_eq!(with.r2, Some(0.8));
        assert_eq!(with.d_prime, None);
    }

    #[test]
    fn species_response_wraps_a_list() {
        let json = r#"{"species":[{"name":"homo_sapiens","display_name":"Human"}]}"#;
        let s: SpeciesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(s.species[0].name, "homo_sapiens");
        assert_eq!(s.species[0].display_name, "Human");
    }

    #[test]
    fn raw_message_fields_become_values() {
        let json = r#"{"name":"rs699","source":"dbSNP","mappings":[{"location":"1:1-1"}]}"#;
        let v: VariationRecord = serde_json::from_str(json).unwrap();
        assert_eq!(v.name, "rs699");
        assert_eq!(v.mappings[0]["location"], "1:1-1");
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --lib types`
Expected: FAIL — `cannot find type PingResponse in this scope`.

- [ ] **Step 3: Write the models in `src/types.rs`**

Start with these, which between them exercise every rule in the table above, then
transcribe the remaining models from `types.go` the same way.

```rust
//! Deserialization models for common Ensembl REST API responses.
//!
//! These cover the shapes the Go port models. They are a convenience, not a
//! requirement: [`crate::Response::json`] accepts any [`serde::Deserialize`]
//! type, including [`serde_json::Value`], so endpoints without a model here are
//! still fully usable.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The response from `/info/ping`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PingResponse {
    /// `1` when the service is up.
    pub ping: i64,
}

/// An archived identifier entry from the `/archive` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ArchiveRecord {
    /// The queried identifier.
    pub id: String,
    /// The latest versioned identifier.
    pub latest: String,
    /// The identifier version.
    pub version: i64,
    /// The Ensembl release this record came from.
    pub release: String,
    /// The assembly name.
    pub assembly: String,
    /// The peptide sequence, when the identifier is a translation.
    pub peptide: Option<String>,
    /// The feature type.
    #[serde(rename = "type")]
    pub kind: String,
    /// Identifiers that may replace a retired one.
    pub possible_replacement: Vec<String>,
    /// Whether this is the current version.
    pub is_current: String,
}

/// A genomic feature returned by the `/lookup` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LookupRecord {
    /// The stable identifier.
    pub id: String,
    /// The feature type, for example `"Gene"` or `"Transcript"`.
    pub object_type: String,
    /// The species name.
    pub species: String,
    /// The display label.
    pub display_name: String,
    /// A free-text description.
    pub description: String,
    /// The biotype, for example `"protein_coding"`.
    pub biotype: String,
    /// The sequence region, typically a chromosome name.
    pub seq_region_name: String,
    /// The start coordinate.
    pub start: i64,
    /// The end coordinate.
    pub end: i64,
    /// The strand, `1` or `-1`.
    pub strand: i64,
    /// The annotation source.
    pub source: String,
    /// The feature version.
    pub version: i64,
    /// The database type.
    pub db_type: String,
    /// The assembly name.
    pub assembly_name: String,
    /// The canonical transcript identifier.
    pub canonical_transcript: String,
    /// Child transcripts, populated by `?expand=1`.
    #[serde(rename = "Transcript")]
    pub transcripts: Vec<LookupRecord>,
    /// The translation, populated by `?expand=1`.
    #[serde(rename = "Translation")]
    pub translation: Option<Box<LookupRecord>>,
    /// Child exons, populated by `?expand=1`.
    #[serde(rename = "Exon")]
    pub exons: Vec<LookupRecord>,
    /// Any additional fields the API returned.
    ///
    /// The Go port declares an equivalent field as `json:"-"`, which means it can
    /// never be populated. Flattening captures what `?expand=1` actually returns.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Linkage-disequilibrium statistics from the `/ld` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LDRecord {
    /// The first variant.
    pub variation1: String,
    /// The second variant.
    pub variation2: String,
    /// The D' statistic.
    pub d_prime: Option<f64>,
    /// The r-squared statistic.
    pub r2: Option<f64>,
}

/// A genetic variant from the `/variation` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct VariationRecord {
    /// The variant name, for example `"rs699"`.
    pub name: String,
    /// The source database.
    pub source: String,
    /// The variant class.
    pub var_class: String,
    /// The most severe consequence term.
    pub most_severe_consequence: String,
    /// The minor allele frequency.
    #[serde(rename = "MAF")]
    pub maf: Option<f64>,
    /// The minor allele.
    pub minor_allele: String,
    /// Supporting evidence terms.
    pub evidence: Vec<String>,
    /// Known synonyms.
    pub synonyms: Vec<String>,
    /// Clinical significance terms.
    pub clinical_significance: Vec<String>,
    /// Genomic mappings.
    pub mappings: Vec<Value>,
    /// Genotype records.
    pub genotypes: Vec<Value>,
    /// Phenotype annotations.
    pub phenotypes: Vec<Value>,
}

/// Species metadata from `/info/species`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SpeciesRecord {
    /// The production name.
    pub name: String,
    /// The human-readable name.
    pub display_name: String,
    /// The NCBI taxonomy identifier.
    pub taxon_id: String,
    /// The species name.
    pub species: String,
    /// The common name.
    pub common_name: String,
    /// The Ensembl division.
    pub division: String,
    /// The Ensembl release.
    pub release: i64,
    /// The assembly name.
    pub assembly: String,
    /// The assembly accession.
    pub accession: String,
    /// Alternative names.
    pub aliases: Vec<String>,
    /// Group memberships.
    pub groups: Vec<String>,
    /// The strain, where applicable.
    pub strain: String,
    /// The strain type.
    pub strain_type: String,
    /// Whether this is the reference genome for its group.
    pub is_reference: i64,
    /// Whether pan-taxonomic comparative data is available.
    pub has_pan_compara: i64,
    /// Whether variation data is available.
    pub has_variations: i64,
    /// Whether peptide comparative data is available.
    pub has_peptide_compara: i64,
    /// Whether whole-genome alignments are available.
    pub has_genome_alignments: i64,
    /// Whether synteny data is available.
    pub has_synteny: i64,
}

/// The wrapper `/info/species` returns.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SpeciesResponse {
    /// The species records.
    pub species: Vec<SpeciesRecord>,
}
```

Then transcribe the remaining models from `types.go`: `SequenceRecord`, `AssemblyInfo`,
`AssemblyRegionInfo`, `HomologyRecord`, `HomologyResponse`, `XrefRecord`,
`MappingRecord`, `VEPRecord`, `BeaconResponse`, `BeaconQueryResponse`.

For `MappingRecord`, Go uses doubly-nested anonymous structs. Name them:

```rust
/// One coordinate in a mapping result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MappedCoordinate {
    /// The sequence region name.
    pub seq_region_name: String,
    /// The start coordinate.
    pub start: i64,
    /// The end coordinate.
    pub end: i64,
    /// The strand.
    pub strand: i64,
    /// The assembly name.
    pub assembly: String,
    /// The coordinate system.
    pub coord_system: String,
}

/// A single original-to-mapped coordinate pair.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Mapping {
    /// The input coordinate.
    pub original: MappedCoordinate,
    /// The projected coordinate.
    pub mapped: MappedCoordinate,
}

/// The result of a `/map` call.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MappingRecord {
    /// The coordinate mappings.
    pub mappings: Vec<Mapping>,
}
```

- [ ] **Step 4: Wire the module into `src/lib.rs`**

```rust
pub mod types;
```

- [ ] **Step 5: Run the tests**

Run: `cargo test --lib types`
Expected: PASS, 8 tests.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
git add src/types.rs src/lib.rs
git commit -m "feat: add domain models for common responses"
```

---

### Tasks 12-14: Domain methods

These three tasks are identical in shape and differ only in which modules they cover.
Do them in order; each is independently reviewable and testable.

| Task | Modules | Methods |
|---|---|---|
| 12 | `archive`, `lookup`, `sequence`, `xrefs`, `mapping`, `overlap`, `comparative` | 29 |
| 13 | `info`, `ontology`, `ld`, `regulation`, `transcript` | 39 |
| 14 | `ga4gh`, `variation`, `vep`, `phenotype` | 38 |

**Files (per task):**
- Create: `src/<module>.rs` for each module in the group
- Modify: `src/lib.rs` (add `mod <module>;` for each — no `pub`, the methods land on
  `Client` via `impl` blocks and need no separate export)
- Modify: `tests/endpoints.rs` (create it in Task 12, extend it in 13 and 14)

**Interfaces:**
- Consumes: `Client::call` and `ENDPOINTS` (Task 10); `RequestOption` (Task 6);
  `Response`, `Result` (Tasks 5, 3); `MockServer` (Task 8)
- Produces: one `pub fn` on `Client` per endpoint in the group. Names are snake_case of
  the Go method names; the endpoint key each one passes to `call` is the string in the
  corresponding Go method's `c.Call(ctx, "...")`.

**Data source:** the matching `.go` file in `goensemblrest`. List a module's methods and
their endpoint keys with:

```bash
cd /Users/gawbul/Documents/Code/goensemblrest
grep -A6 '^func (c \*Client)' lookup.go | grep -E '^func|c\.Call'
```

**The four method shapes.** Every one of the 106 methods is one of these. Transcribe
each Go method into the matching shape:

```rust
//! Genomic feature lookup.

use serde_json::json;

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    // Shape 1 — GET, one path parameter.
    /// Finds the species and database for a single identifier.
    pub fn get_lookup_by_id(&self, id: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getLookupById", &[("id", id)], None, opts)
    }

    // Shape 2 — GET, two path parameters.
    /// Finds the species and database for a symbol in a linked external database.
    pub fn get_lookup_by_symbol(
        &self,
        species: &str,
        symbol: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getLookupBySymbol", &[("species", species), ("symbol", symbol)], None, opts)
    }

    // Shape 3 — POST, body only.
    /// Finds the species and database for several identifiers.
    pub fn get_lookup_by_multiple_ids(
        &self,
        ids: &[&str],
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getLookupByMultipleIds", &[], Some(&json!({ "ids": ids })), opts)
    }

    // Shape 4 — POST, one path parameter plus a body.
    /// Finds the species and database for a set of symbols.
    pub fn get_lookup_by_multiple_symbols(
        &self,
        species: &str,
        symbols: &[&str],
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getLookupByMultipleSymbols",
            &[("species", species)],
            Some(&json!({ "symbols": symbols })),
            opts,
        )
    }
}
```

**The body key is per-endpoint and must be copied from the Go source, not guessed.**
`getArchiveByMultipleIds` sends `{"id": [...]}` while `getLookupByMultipleIds` sends
`{"ids": [...]}`. That asymmetry is real Ensembl behaviour. It is also the value in the
endpoint's `post_parameters`, so `ENDPOINTS` is the cross-check.

- [ ] **Step 1: Enumerate the group's methods from the Go source**

```bash
cd /Users/gawbul/Documents/Code/goensemblrest
for f in archive lookup sequence xrefs mapping overlap comparative; do
  echo "=== $f ==="
  grep -A8 '^func (c \*Client)' $f.go | grep -E '^func \(c|c\.Call\(ctx|"[a-z_]+": *[a-z]'
done
```

(Substitute the group's modules for Tasks 13 and 14.)

- [ ] **Step 2: Write the failing tests in `tests/endpoints.rs`**

One test per method. In Task 12 create the file with this header; in Tasks 13 and 14
append to it.

```rust
//! Offline tests for every typed endpoint method.
//!
//! Each test asserts the HTTP method, resolved path and request body that a
//! typed method produces, against the std-only mock server.

mod common;

use common::mock::MockServer;
use ensemblrest::Client;

/// A client pointed at a mock server returning `{}` once.
fn client(server: &MockServer) -> Client {
    Client::builder().base_url(server.base_url()).build().unwrap()
}

#[test]
fn get_lookup_by_id() {
    let server = MockServer::with_json(200, r#"{"id":"ENSG00000157764"}"#);
    client(&server).get_lookup_by_id("ENSG00000157764", &[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/lookup/id/ENSG00000157764");
}

#[test]
fn get_lookup_by_symbol() {
    let server = MockServer::with_json(200, "{}");
    client(&server).get_lookup_by_symbol("homo_sapiens", "BRAF", &[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/lookup/symbol/homo_sapiens/BRAF");
}

#[test]
fn get_lookup_by_multiple_ids() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_lookup_by_multiple_ids(&["ENSG00000157764", "ENSG00000248378"], &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/lookup/id");
    assert_eq!(req.json()["ids"][0], "ENSG00000157764");
    assert_eq!(req.json()["ids"][1], "ENSG00000248378");
}

#[test]
fn get_archive_by_multiple_ids_uses_the_id_key_not_ids() {
    // Copied from the Go source: archive is the odd one out.
    let server = MockServer::with_json(200, "[]");
    client(&server).get_archive_by_multiple_ids(&["ENSG00000157764"], &[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.path(), "/archive/id");
    assert_eq!(req.json()["id"][0], "ENSG00000157764");
    assert!(req.json().get("ids").is_none(), "archive must send 'id', not 'ids'");
}

#[test]
fn get_sequence_by_region_preserves_colons() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_sequence_by_region("homo_sapiens", "X:1000000..1000100:1", &[])
        .unwrap();

    assert_eq!(
        server.only_request().path(),
        "/sequence/region/homo_sapiens/X:1000000..1000100:1"
    );
}

// ... one test per remaining method in this task's group ...
```

- [ ] **Step 3: Run to verify they fail**

Run: `cargo test --test endpoints`
Expected: FAIL — `no method named get_lookup_by_id found for struct Client`.

- [ ] **Step 4: Write the module implementations**

Create each `src/<module>.rs` in the group, transcribing every Go method into one of the
four shapes above. Copy each Go doc comment as the Rust doc comment.

- [ ] **Step 5: Declare the modules in `src/lib.rs`**

```rust
mod archive;
mod comparative;
mod lookup;
mod mapping;
mod overlap;
mod sequence;
mod xrefs;
```

- [ ] **Step 6: Run the tests**

Run: `cargo test --test endpoints`
Expected: PASS, one test per method in the group.

- [ ] **Step 7: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
git add src/ tests/endpoints.rs
git commit -m "feat: add archive, lookup, sequence, xrefs, mapping, overlap and comparative methods"
```

(Adjust the module list and commit message per task.)

---

### Task 15: Endpoint/method parity test

**Files:**
- Create: `tests/parity.rs`

**Interfaces:**
- Consumes: `ENDPOINTS` (Task 10); the domain modules (Tasks 12-14)
- Produces: nothing consumed by later tasks

Rust cannot enumerate a type's methods at runtime, so this test reads the domain module
sources and matches every `self.call("...")` against `ENDPOINTS`. Cargo runs integration
tests with the crate root as the working directory, so `src/` resolves. This is the
guard the Go port relies on a manual procedure for, and it is what keeps the
106↔106 invariant true as endpoints are added.

- [ ] **Step 1: Write the test in `tests/parity.rs`**

```rust
//! Enforces the 106-endpoint/106-method invariant.

use ensemblrest::endpoints::ENDPOINTS;
use std::collections::BTreeMap;
use std::fs;

/// Modules that define typed endpoint methods.
///
/// `endpoints.rs` is excluded because it defines the table and the generic
/// `call`, not typed methods.
fn domain_sources() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for entry in fs::read_dir("src").expect("read src/") {
        let path = entry.expect("dir entry").path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        if matches!(
            name.as_str(),
            "lib.rs" | "endpoints.rs" | "client.rs" | "request.rs" | "response.rs"
                | "error.rs" | "options.rs" | "ratelimit.rs" | "encoding.rs" | "types.rs"
        ) {
            continue;
        }
        out.push((name, fs::read_to_string(&path).expect("read source")));
    }
    out
}

/// Extracts every endpoint name passed to `self.call("...")`.
fn called_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = source;
    while let Some(idx) = rest.find("self.call(\"") {
        rest = &rest[idx + "self.call(\"".len()..];
        if let Some(end) = rest.find('"') {
            names.push(rest[..end].to_string());
            rest = &rest[end..];
        }
    }
    names
}

#[test]
fn there_are_sixteen_domain_modules() {
    let mut modules: Vec<String> = domain_sources().into_iter().map(|(n, _)| n).collect();
    modules.sort();
    assert_eq!(modules.len(), 16, "expected 16 domain modules, found {modules:?}");
}

#[test]
fn every_endpoint_is_reached_by_exactly_one_typed_method() {
    let mut counts: BTreeMap<&str, usize> = ENDPOINTS.iter().map(|e| (e.name, 0)).collect();

    for (_, source) in domain_sources() {
        for name in called_names(&source) {
            if let Some(count) = counts.get_mut(name.as_str()) {
                *count += 1;
            }
        }
    }

    let missing: Vec<&str> = counts.iter().filter(|(_, &c)| c == 0).map(|(n, _)| *n).collect();
    assert!(missing.is_empty(), "endpoints with no typed method: {missing:?}");

    let duplicated: Vec<&str> = counts.iter().filter(|(_, &c)| c > 1).map(|(n, _)| *n).collect();
    assert!(duplicated.is_empty(), "endpoints with more than one method: {duplicated:?}");
}

#[test]
fn every_typed_method_targets_an_endpoint_that_exists() {
    let known: Vec<&str> = ENDPOINTS.iter().map(|e| e.name).collect();
    let mut unknown = Vec::new();

    for (module, source) in domain_sources() {
        for name in called_names(&source) {
            if !known.contains(&name.as_str()) {
                unknown.push(format!("{module}: {name}"));
            }
        }
    }

    assert!(unknown.is_empty(), "methods calling names absent from ENDPOINTS: {unknown:?}");
}

#[test]
fn the_method_count_matches_the_endpoint_count() {
    let total: usize = domain_sources()
        .iter()
        .map(|(_, s)| called_names(s).len())
        .sum();
    assert_eq!(total, ENDPOINTS.len(), "106 endpoints require 106 typed methods");
}
```

- [ ] **Step 2: Run the test**

Run: `cargo test --test parity`
Expected: PASS, 4 tests. A failure names the exact endpoints that are missing,
duplicated or misspelled.

- [ ] **Step 3: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
git add tests/parity.rs
git commit -m "test: enforce endpoint/method parity"
```

---

### Task 16: Live tests and example

**Files:**
- Create: `tests/live.rs`, `examples/basic.rs`
- Modify: `src/client.rs` (restore the doctest deferred in Task 7)

**Interfaces:**
- Consumes: the full public API
- Produces: nothing consumed by later tasks

- [ ] **Step 1: Write `tests/live.rs`**

```rust
//! Smoke tests against the live Ensembl REST API.
//!
//! These are `#[ignore]`d and additionally gated on `ENSEMBL_LIVE_TESTS=1`, so
//! neither `cargo test` nor `cargo test -- --ignored` touches the network
//! unless asked. Run with:
//!
//! ```text
//! ENSEMBL_LIVE_TESTS=1 cargo test --test live -- --ignored
//! ```

use ensemblrest::types::{LookupRecord, PingResponse, SpeciesResponse};
use ensemblrest::options::content_type;
use ensemblrest::Client;

/// Returns a live client, or `None` when live testing is not enabled.
fn live_client() -> Option<Client> {
    if std::env::var("ENSEMBL_LIVE_TESTS").as_deref() != Ok("1") {
        eprintln!("skipping: set ENSEMBL_LIVE_TESTS=1 to run live tests");
        return None;
    }
    Some(Client::new().expect("build client"))
}

#[test]
#[ignore = "requires network access and ENSEMBL_LIVE_TESTS=1"]
fn ping_reports_the_service_is_up() {
    let Some(c) = live_client() else { return };
    let ping: PingResponse = c.get_info_ping(&[]).unwrap().json().unwrap();
    assert_eq!(ping.ping, 1);
}

#[test]
#[ignore = "requires network access and ENSEMBL_LIVE_TESTS=1"]
fn lookup_returns_a_known_stable_gene() {
    let Some(c) = live_client() else { return };
    // BRAF is stable reference data and safe to assert on.
    let rec: LookupRecord = c.get_lookup_by_id("ENSG00000157764", &[]).unwrap().json().unwrap();
    assert_eq!(rec.id, "ENSG00000157764");
    assert_eq!(rec.display_name, "BRAF");
    assert_eq!(rec.species, "homo_sapiens");
}

#[test]
#[ignore = "requires network access and ENSEMBL_LIVE_TESTS=1"]
fn species_list_includes_humans() {
    let Some(c) = live_client() else { return };
    let species: SpeciesResponse = c.get_info_species(&[]).unwrap().json().unwrap();
    assert!(species.species.iter().any(|s| s.name == "homo_sapiens"));
}

#[test]
#[ignore = "requires network access and ENSEMBL_LIVE_TESTS=1"]
fn sequence_can_be_fetched_as_fasta() {
    let Some(c) = live_client() else { return };
    let fasta = c
        .get_sequence_by_region("homo_sapiens", "X:1000000..1000100:1",
                                &[content_type("text/x-fasta")])
        .unwrap()
        .text()
        .unwrap();
    assert!(fasta.starts_with('>'), "expected FASTA, got: {}", &fasta[..fasta.len().min(80)]);
}

#[test]
#[ignore = "requires network access and ENSEMBL_LIVE_TESTS=1"]
fn an_unknown_identifier_is_a_clean_error() {
    let Some(c) = live_client() else { return };
    let err = c.get_lookup_by_id("ENSG00000000000", &[]).unwrap_err();
    assert!(err.is_bad_request() || err.is_not_found(), "got {err}");
}

#[test]
#[ignore = "requires network access and ENSEMBL_LIVE_TESTS=1"]
fn rate_limit_telemetry_is_reported() {
    let Some(c) = live_client() else { return };
    c.get_info_ping(&[]).unwrap();
    assert!(c.rate_limit().limit.is_some(), "Ensembl sends X-RateLimit-Limit");
}
```

- [ ] **Step 2: Run the live tests**

Run: `ENSEMBL_LIVE_TESTS=1 cargo test --test live -- --ignored`
Expected: PASS, 6 tests. If Ensembl is down, they fail on network errors rather than
assertions; this is expected and is why CI marks them `continue-on-error`.

- [ ] **Step 3: Verify they are skipped by default**

Run: `cargo test --test live`
Expected: 6 tests reported as ignored, no network traffic.

- [ ] **Step 4: Write `examples/basic.rs`**

```rust
//! A tour of the `ensemblrest` client.
//!
//! Run with: `cargo run --example basic`

use ensemblrest::options::{content_type, query};
use ensemblrest::types::{LookupRecord, PingResponse, SpeciesResponse};
use ensemblrest::{ApiErrorKind, Client};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("ensemblrest-example/1.0")
        .build()?;

    let ping: PingResponse = client.get_info_ping(&[])?.json()?;
    println!("service up: {}", ping.ping == 1);

    let braf: LookupRecord = client.get_lookup_by_id("ENSG00000157764", &[])?.json()?;
    println!(
        "{} ({}) on {}:{}-{}",
        braf.display_name, braf.biotype, braf.seq_region_name, braf.start, braf.end
    );

    // Expanded lookups return nested transcripts.
    let expanded: LookupRecord = client
        .get_lookup_by_id("ENSG00000157764", &[query("expand", "1")])?
        .json()?;
    println!("transcripts: {}", expanded.transcripts.len());

    // Non-JSON content types come back through .text().
    let fasta = client
        .get_sequence_by_region("homo_sapiens", "X:1000000..1000100:1",
                                &[content_type("text/x-fasta")])?
        .text()?;
    println!("fasta header: {}", fasta.lines().next().unwrap_or_default());

    // POST endpoints take slices.
    let records: Vec<LookupRecord> = client
        .get_archive_by_multiple_ids(&["ENSG00000157764", "ENSG00000248378"], &[])?
        .json()?;
    println!("archive records: {}", records.len());

    let species: SpeciesResponse = client.get_info_species(&[])?.json()?;
    println!("species known: {}", species.species.len());

    // Errors carry the status and the server's message.
    match client.get_lookup_by_id("NOT_A_REAL_ID", &[]) {
        Err(e) if e.api_kind() == Some(ApiErrorKind::BadRequest) => {
            println!("expected failure: {e}");
        }
        Err(e) => println!("unexpected failure: {e}"),
        Ok(_) => println!("unexpectedly succeeded"),
    }

    // Dynamic dispatch by endpoint name, for parity with the Go and Python ports.
    let v: serde_json::Value = client
        .call("getLookupById", &[("id", "ENSG00000157764")], None, &[])?
        .json()?;
    println!("dynamic dispatch: {}", v["display_name"]);

    if let Some(remaining) = client.rate_limit().remaining {
        println!("requests remaining: {remaining}");
    }

    Ok(())
}
```

- [ ] **Step 5: Run the example**

Run: `cargo run --example basic`
Expected: prints each section without error, given network access.

- [ ] **Step 6: Restore the deferred doctest**

In `src/client.rs`, change the `Client` doctest back from ```ignore``` to ```no_run```
now that `get_info_species` exists.

Run: `cargo test --doc`
Expected: PASS, all doctests compile.

- [ ] **Step 7: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
git add tests/live.rs examples/basic.rs src/client.rs
git commit -m "test: add live smoke tests and a runnable example"
```

---

### Task 17: Documentation and automation

**Files:**
- Create: `README.md`, `Makefile`, `AGENTS.md`, `CLAUDE.md`
- Modify: `src/lib.rs` (crate-level docs)

**Interfaces:**
- Consumes: the full public API
- Produces: nothing consumed by later tasks

- [ ] **Step 1: Write the `Makefile`**

```makefile
.PHONY: all build test test-live test-coverage lint format example clean

all: lint test build

build:
	cargo build --all-targets

test:
	cargo test

test-live:
	ENSEMBL_LIVE_TESTS=1 cargo test --test live -- --ignored

# Requires cargo-llvm-cov: cargo install cargo-llvm-cov
test-coverage:
	cargo llvm-cov --html --open

lint:
	cargo clippy --all-targets -- -D warnings

format:
	cargo fmt

format-check:
	cargo fmt --check

example:
	cargo run --example basic

clean:
	cargo clean
```

There is deliberately no `test-race` target. Go's `-race` detector has no Rust
equivalent because `Send` and `Sync` are checked at compile time; `make test` covers it.

- [ ] **Step 2: Write the crate-level documentation in `src/lib.rs`**

Replace the two-line header from Task 1 with:

```rust
//! A Rust client library for the [Ensembl REST API](https://rest.ensembl.org/).
//!
//! This crate is a port of [`goensemblrest`](https://github.com/gawbul/goensemblrest),
//! itself a port of [`pyEnsemblRest`](https://github.com/gawbul/pyEnsemblRest), and
//! covers all 106 endpoints.
//!
//! # Quickstart
//!
//! ```no_run
//! use ensemblrest::types::LookupRecord;
//! use ensemblrest::Client;
//!
//! # fn main() -> ensemblrest::Result<()> {
//! let client = Client::new()?;
//! let braf: LookupRecord = client.get_lookup_by_id("ENSG00000157764", &[])?.json()?;
//! println!("{} is a {}", braf.display_name, braf.biotype);
//! # Ok(())
//! # }
//! ```
//!
//! # Decoding responses
//!
//! Every endpoint method returns a [`Response`]. Decode it with [`Response::json`]
//! for JSON, or [`Response::text`] for the formats Ensembl serves as text.
//!
//! ```no_run
//! use ensemblrest::options::content_type;
//! use ensemblrest::Client;
//!
//! # fn main() -> ensemblrest::Result<()> {
//! let client = Client::new()?;
//! let fasta = client
//!     .get_sequence_by_id("ENSG00000157764", &[content_type("text/x-fasta")])?
//!     .text()?;
//! # Ok(())
//! # }
//! ```
//!
//! `json` accepts any [`serde::Deserialize`] type, so endpoints without a model in
//! [`types`] still work via [`serde_json::Value`].
//!
//! # Configuration
//!
//! ```no_run
//! use ensemblrest::Client;
//! use std::time::Duration;
//!
//! # fn main() -> ensemblrest::Result<()> {
//! let client = Client::builder()
//!     .timeout(Duration::from_secs(30))
//!     .rate_limit(15, Duration::from_secs(1))
//!     .max_attempts(5)
//!     .user_agent("my-tool/1.0")
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! [`Client`] is cheap to clone and shares one rate limiter and connection pool
//! across clones, so a cloned client observes one global rate limit.
//!
//! # Errors
//!
//! ```no_run
//! use ensemblrest::{ApiErrorKind, Client};
//!
//! # fn main() -> ensemblrest::Result<()> {
//! let client = Client::new()?;
//! match client.get_lookup_by_id("NOT_A_REAL_ID", &[]) {
//!     Ok(response) => println!("{}", response.status()),
//!     Err(e) if e.is_not_found() => println!("no such identifier"),
//!     Err(e) if e.api_kind() == Some(ApiErrorKind::RateLimit) => println!("rate limited"),
//!     Err(e) => println!("failed: {e}"),
//! }
//! # Ok(())
//! # }
//! ```
#![doc(html_root_url = "https://docs.rs/ensemblrest")]
```

- [ ] **Step 3: Write `README.md`**

Include, in this order: a one-paragraph description naming the Go and Python ports; an
install snippet (`cargo add ensemblrest`); the quickstart, decoding, configuration and
error-handling examples from the crate docs; a note that the crate has three
dependencies and zero dev-dependencies; the `Makefile` target table; and the full
endpoint catalog.

Generate the catalog table from the implemented table rather than writing it by hand:

```bash
cat > /tmp/catalog.rs <<'EOF'
fn main() {
    println!("| Endpoint | Method | Path |");
    println!("|---|---|---|");
    for e in ensemblrest::endpoints::ENDPOINTS {
        let m = match e.method {
            ensemblrest::endpoints::Method::Get => "GET",
            ensemblrest::endpoints::Method::Post => "POST",
        };
        println!("| `{}` | {} | `{}` | ", e.name, m, e.url);
    }
}
EOF
mkdir -p examples && cp /tmp/catalog.rs examples/catalog.rs
cargo run --quiet --example catalog >> README.md
rm examples/catalog.rs
```

- [ ] **Step 4: Write `AGENTS.md`**

Adapt `/Users/gawbul/Documents/Code/goensemblrest/AGENTS.md`, replacing the Go-specific
sections. It must cover: the three-dependency and zero-dev-dependency rule; the
camelCase endpoint-key rule; the colon-preservation invariant; the procedure for adding
an endpoint (add to `ENDPOINTS`, add the typed method, add a test in `tests/endpoints.rs`,
run `cargo test --test parity`); and the `make` targets.

- [ ] **Step 5: Point `CLAUDE.md` at it**

```markdown
# CLAUDE.md

See [AGENTS.md](AGENTS.md) for the full developer and agent guide.
```

- [ ] **Step 6: Verify the docs build cleanly**

```bash
cargo doc --no-deps
cargo test --doc
```
Expected: no warnings, all doctests pass.

- [ ] **Step 7: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
git add README.md Makefile AGENTS.md CLAUDE.md src/lib.rs
git commit -m "docs: add README, agent guide and Makefile"
```

---

### Task 18: CI workflows

**Files:**
- Create: `.github/workflows/pull_request.yaml`, `.github/workflows/nightly.yaml`, `.github/workflows/push_tag.yaml`

**Interfaces:**
- Consumes: the `make` targets from Task 17
- Produces: nothing consumed by later tasks

**Prerequisite:** `cargo publish` needs a `CARGO_REGISTRY_TOKEN` repository secret. Steve
is supplying this; the release job fails at the publish step without it, which is the
intended behaviour rather than a silent skip.

- [ ] **Step 1: Write `.github/workflows/pull_request.yaml`**

```yaml
name: Pull Request

on:
  pull_request:
    paths: ['**.rs', 'Cargo.toml', '.github/workflows/**']
  push:
    branches: [main]
    paths: ['**.rs', 'Cargo.toml', '.github/workflows/**']

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: '1.98'
          components: rustfmt, clippy

      - uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --check

      - name: Lint
        run: cargo clippy --all-targets -- -D warnings

      - name: Verify the dependency budget
        run: |
          direct=$(cargo tree --depth 1 --prefix none | tail -n +2 | grep -c . || true)
          echo "direct dependencies: $direct"
          test "$direct" -eq 3 || { echo "expected exactly 3 direct dependencies"; exit 1; }

      - name: Test
        run: cargo test --all-targets

      - name: Doctests
        run: cargo test --doc

      - name: Build the example
        run: cargo build --examples

  live:
    runs-on: ubuntu-latest
    continue-on-error: true
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: '1.98'
      - uses: Swatinem/rust-cache@v2
      - name: Live smoke tests
        env:
          ENSEMBL_LIVE_TESTS: '1'
        run: cargo test --test live -- --ignored
```

The `live` job is `continue-on-error` so an Ensembl outage cannot block a PR.

- [ ] **Step 2: Write `.github/workflows/nightly.yaml`**

```yaml
name: Nightly API Drift Check

on:
  schedule:
    - cron: '0 3 * * *'
  workflow_dispatch:

jobs:
  drift:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: '1.98'
      - uses: Swatinem/rust-cache@v2
      - name: Live tests against rest.ensembl.org
        env:
          ENSEMBL_LIVE_TESTS: '1'
        run: cargo test --test live -- --ignored --nocapture
```

Unlike the PR job this one is allowed to fail: a failure here is the signal that Ensembl
changed something upstream.

- [ ] **Step 3: Write `.github/workflows/push_tag.yaml`**

```yaml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  release:
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: '1.98'
          components: rustfmt, clippy

      - uses: Swatinem/rust-cache@v2

      - name: Verify the tag matches the crate version
        run: |
          tag="${GITHUB_REF_NAME#v}"
          crate=$(cargo metadata --no-deps --format-version 1 \
            | sed -n 's/.*"name":"ensemblrest","version":"\([^"]*\)".*/\1/p')
          echo "tag=$tag crate=$crate"
          test "$tag" = "$crate" || { echo "tag $tag does not match Cargo.toml $crate"; exit 1; }

      - name: Full test suite
        run: |
          cargo fmt --check
          cargo clippy --all-targets -- -D warnings
          cargo test --all-targets
          cargo test --doc

      - name: Publish to crates.io
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: cargo publish

      - name: Create the GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true
```

- [ ] **Step 4: Validate the workflow syntax locally**

```bash
for f in .github/workflows/*.yaml; do
  python3 -c "import sys,yaml; yaml.safe_load(open('$f')); print('$f ok')"
done
```
Expected: three `ok` lines. (`yaml` ships with most Python installs; if absent, rely on
GitHub's own validation on push.)

- [ ] **Step 5: Run the full local suite one last time**

```bash
make all
make test-live
```
Expected: lint clean, all tests pass, example builds, live tests pass.

- [ ] **Step 6: Commit**

```bash
git add .github/workflows
git commit -m "ci: add PR, nightly drift and release workflows"
```

---

## Completion checklist

Verify all of these before declaring the port done:

- [ ] `cargo tree --depth 1` shows exactly three dependencies
- [ ] `[dev-dependencies]` is empty
- [ ] `cargo test` passes, including `tests/parity.rs`
- [ ] `ENDPOINTS.len() == 106` and the parity test finds 106 typed methods
- [ ] The endpoint-name diff against `goensemblrest/endpoints.go` prints `IDENTICAL`
- [ ] `cargo clippy --all-targets -- -D warnings` is clean
- [ ] `cargo fmt --check` is clean
- [ ] `cargo doc --no-deps` emits no warnings
- [ ] `ENSEMBL_LIVE_TESTS=1 cargo test --test live -- --ignored` passes
- [ ] `cargo run --example basic` runs end to end
- [ ] `cargo publish --dry-run` succeeds
