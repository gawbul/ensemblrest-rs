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
// TODO(Task 16): restore to no_run once get_info_species exists
/// ```ignore
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

    /// Shrinks the rate-limit window so retry backoff is fast in tests.
    ///
    /// Backoff is `attempt * wall_time * 2`, so a 5 ms window keeps a
    /// four-retry sequence under 100 ms.
    #[doc(hidden)]
    pub fn wall_time_for_test(self) -> Self {
        self.rate_limit(1000, Duration::from_millis(5))
    }

    /// Supplies a pre-configured `ureq` agent, bypassing the timeout setting.
    ///
    /// The agent must be built with `http_status_as_error(false)`, otherwise
    /// non-2xx responses surface as transport errors and the retry
    /// classification in this crate cannot see their status codes. This is
    /// enforced: [`build`](ClientBuilder::build) rejects a supplied agent
    /// that doesn't have the flag set with [`Error::InvalidConfig`].
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
            return Err(Error::InvalidConfig(
                "timeout must be greater than zero".into(),
            ));
        }
        if self.reqs_per_sec == 0 {
            return Err(Error::InvalidConfig("reqs_per_sec must be positive".into()));
        }
        if self.wall_time.is_zero() {
            return Err(Error::InvalidConfig(
                "rate limit window must be positive".into(),
            ));
        }
        if self.max_attempts == 0 {
            return Err(Error::InvalidConfig(
                "max_attempts must be at least 1".into(),
            ));
        }
        if let Some(supplied) = &self.agent
            && supplied.config().http_status_as_error()
        {
            return Err(Error::InvalidConfig(
                "supplied agent must be built with http_status_as_error(false); \
                 without it ureq turns non-2xx responses into transport errors and \
                 this client cannot classify or retry them"
                    .into(),
            ));
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
        let c = Client::builder()
            .base_url("https://example.org///")
            .build()
            .unwrap();
        assert_eq!(c.base_url(), "https://example.org");
    }

    #[test]
    fn builder_rejects_zero_timeout() {
        let err = Client::builder()
            .timeout(Duration::ZERO)
            .build()
            .unwrap_err();
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
        assert_eq!(
            c.inner.headers,
            vec![("X-Custom".to_string(), "value".to_string())]
        );
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

    #[test]
    fn builder_rejects_a_supplied_agent_missing_http_status_as_error_false() {
        // ureq's default is `http_status_as_error(true)`; a caller who forgets
        // to turn it off would otherwise get non-2xx responses silently
        // converted to transport errors, hiding the status code and body
        // this crate's retry logic classifies on.
        let agent: ureq::Agent = ureq::Agent::config_builder().build().into();
        let err = Client::builder().agent(agent).build().unwrap_err();
        assert!(matches!(err, Error::InvalidConfig(_)), "got {err:?}");
    }

    #[test]
    fn builder_accepts_a_supplied_agent_with_http_status_as_error_false() {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build()
            .into();
        let c = Client::builder().agent(agent).build().unwrap();
        assert_eq!(c.base_url(), crate::DEFAULT_BASE_URL);
    }
}
