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
//! # GA4GH query structs
//!
//! The GA4GH search endpoints take a body of optional filters, most of them the
//! same type as their neighbours. Each such endpoint therefore takes a named
//! `Ga4gh*Query` struct rather than a row of positional `Option`s, so a
//! transposed pair cannot compile into a silently wrong query. Fields left
//! `None` are omitted from the request body.
//!
//! ```no_run
//! use ensemblrest::{Client, Ga4ghReferencesQuery};
//!
//! # fn main() -> ensemblrest::Result<()> {
//! let client = Client::new()?;
//! let references = client.search_ga4gh_references(
//!     &Ga4ghReferencesQuery {
//!         reference_set_id: Some("GRCh38"),
//!         accession: Some("GCA_000001405"),
//!         page_size: Some(10),
//!         ..Default::default()
//!     },
//!     &[],
//! )?;
//! # Ok(())
//! # }
//! ```
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
// `Error` is ~136 bytes, dominated by ApiError's String, Vec<u8> and RateLimitInfo.
// Every fallible function in this crate accompanies a network round trip, so the
// cost of moving it is immaterial and boxing it would change a public API shape
// that callers pattern-match on.
#![allow(clippy::result_large_err)]

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

/// The crate's minimum supported Rust version, from `rust-version` in
/// `Cargo.toml`.
///
/// `None` when built by a toolchain that does not set
/// `CARGO_PKG_RUST_VERSION`, or when `Cargo.toml` declares no `rust-version`.
pub const MSRV: Option<&str> = option_env!("CARGO_PKG_RUST_VERSION");

/// Returns the default `User-Agent` header value.
///
/// The Rust version is taken from [`MSRV`] rather than hardcoded, so bumping
/// `rust-version` in `Cargo.toml` cannot leave a stale number on the wire. If
/// the toolchain does not supply one, the version is left out altogether
/// rather than guessed.
pub fn default_user_agent() -> String {
    const HOME: &str = "+https://github.com/gawbul/ensemblrest-rs";
    match MSRV {
        Some(rust) if !rust.is_empty() => format!("ensemblrest/{VERSION} (Rust {rust}; {HOME})"),
        _ => format!("ensemblrest/{VERSION} ({HOME})"),
    }
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

pub mod response;

pub use response::Response;

pub mod options;

pub use options::{RequestOption, content_type, header, query};

pub mod client;

pub use client::{Client, ClientBuilder};

pub(crate) mod request;

pub mod endpoints;

pub mod types;

mod archive;
mod comparative;
mod ga4gh;

pub use ga4gh::{
    Ga4ghBeaconQuery, Ga4ghCallsetQuery, Ga4ghFeaturesQuery, Ga4ghFeaturesetsQuery,
    Ga4ghReferencesQuery, Ga4ghReferencesetsQuery, Ga4ghVariantAnnotationsQuery,
    Ga4ghVariantAnnotationsetsQuery, Ga4ghVariantsQuery, Ga4ghVariantsetsQuery,
};

mod info;
mod ld;
mod lookup;
mod mapping;
mod ontology;
mod overlap;
mod phenotype;
mod regulation;
mod sequence;
mod transcript;
mod variation;
mod vep;
mod xrefs;

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
        // The Rust version must track Cargo.toml, never a hardcoded literal.
        if let Some(rust) = MSRV {
            assert!(ua.contains(&format!("Rust {rust}")), "got {ua}");
        }
    }
}
