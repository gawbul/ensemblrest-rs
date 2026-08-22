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

/// Re-export of the `serde_json` version this crate was built against.
///
/// Use this to guarantee your `serde_json::Value` types match the ones
/// produced by this crate.
pub use serde_json;

pub mod ratelimit;

pub use ratelimit::RateLimitInfo;

pub mod error;

pub use error::{ApiError, ApiErrorKind, Error, Result};

pub(crate) mod encoding;

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
