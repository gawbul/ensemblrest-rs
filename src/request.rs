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

/// Upper bound, in seconds, on a server-supplied `Retry-After` this client
/// will honour.
///
/// This does two jobs. **Safety:** `RateLimiter::update_from_headers` parses
/// `Retry-After` with `f64::from_str`, which happily accepts `inf`, values
/// that overflow to infinity (`1e400`) and values far beyond `u64::MAX` --
/// every one of which makes `Duration::from_secs_f64` *panic*. A garbled or
/// hostile header must not abort the caller's thread, so the value is clamped
/// before it is turned into a `Duration`. **Liveness:** the retry sleep is not
/// cancellable (see the design spec's divergence 3), so this also bounds how
/// long a single call can park a caller's thread on one server-dictated wait.
const MAX_RETRY_AFTER_SECS: f64 = 300.0;

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

/// Sets `name` to exactly one value on `r`, replacing any prior occurrence.
///
/// `ureq`'s builder-level `.header()` always *appends* to the underlying
/// `HeaderMap` (it calls `HeaderMap::append`, never `insert`), which matches
/// Go's `Header.Add`. The fixed headers and per-request
/// [`crate::RequestOption::Header`]s need Go's `Header.Set` semantics
/// instead: a caller-set `Accept` must not survive alongside a
/// `content_type(...)` override, and this crate's own `User-Agent` must
/// always win over a caller-supplied one. `HeaderMap::insert`, reached via
/// `headers_mut()`, gives real replace semantics — it drops every existing
/// value under `name` and stores just the new one.
fn set_header<T>(
    r: &mut ureq::RequestBuilder<T>,
    name: &str,
    value: &str,
) -> std::result::Result<(), ureq::Error> {
    let Some(map) = r.headers_mut() else {
        // The builder already captured an earlier header error (invalid
        // name/value from a prior `.header()` call); that error surfaces
        // when `.call()`/`.send()` runs, so there is nothing to insert into.
        return Ok(());
    };
    let name = ureq::http::HeaderName::try_from(name).map_err(ureq::http::Error::from)?;
    let value = ureq::http::HeaderValue::try_from(value).map_err(ureq::http::Error::from)?;
    map.insert(name, value);
    Ok(())
}

/// Applies client headers, per-request headers, then the fixed headers.
///
/// Client-level headers accumulate (Go's `Header.Add`); per-request and
/// fixed headers each replace any prior value under the same name (Go's
/// `Header.Set`), so the fixed trio always wins regardless of call order.
/// Generic over the `ureq` builder typestate so one function serves both
/// `RequestBuilder<WithoutBody>` (GET) and `RequestBuilder<WithBody>` (POST).
fn apply_headers<T>(
    mut r: ureq::RequestBuilder<T>,
    client: &Client,
    cfg: &RequestConfig<'_>,
) -> std::result::Result<ureq::RequestBuilder<T>, ureq::Error> {
    for (k, v) in &client.inner.headers {
        r = r.header(k, v);
    }
    for (k, v) in &cfg.headers {
        set_header(&mut r, k, v)?;
    }
    // Applied last so they win over caller-supplied values, matching Go.
    set_header(&mut r, "User-Agent", &client.inner.user_agent)?;
    set_header(&mut r, "Content-Type", cfg.content_type)?;
    set_header(&mut r, "Accept", cfg.content_type)?;
    Ok(r)
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
            Method::Get => apply_headers(self.inner.agent.get(url), self, cfg)?.call(),
            Method::Post => {
                apply_headers(self.inner.agent.post(url), self, cfg)?.send(body.unwrap_or(&[]))
            }
        }
    }

    /// Computes how long to wait before the next attempt.
    ///
    /// `retry_after` is the *merged*, client-global `Retry-After`, not the one
    /// this attempt's response happened to send, and that is deliberate: it is
    /// sticky, so once a 429 supplies `Retry-After`, a later attempt that fails
    /// without one (e.g. a 503) still sleeps for it. That looks like a bug but
    /// isn't one — `goensemblrest/ratelimit.go` behaves identically, so this is a
    /// faithful port, not a divergence to "fix". The per-response value that
    /// [`Response::rate_limit`] and [`ApiError::rate_limit`] expose is a
    /// different, non-sticky view; see [`crate::ratelimit::parse_headers`].
    fn backoff(&self, attempt: u32, retry_after: Option<f64>) -> Duration {
        if let Some(retry_after) = retry_after
            && retry_after > 0.0
        {
            // `retry_after > 0.0` already excludes NaN and negatives; the
            // clamp excludes infinity and anything that would overflow a
            // `Duration`. See `MAX_RETRY_AFTER_SECS`.
            return Duration::from_secs_f64(retry_after.min(MAX_RETRY_AFTER_SECS));
        }
        // Saturating: `wall_time` is caller-configured and an absurd window
        // (say `Duration::MAX / 2`) would otherwise overflow-panic here.
        self.inner
            .wall_time
            .saturating_mul(2)
            .saturating_mul(attempt)
            .max(Duration::from_millis(10))
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

            // The sticky, client-global `Retry-After` that drives backoff. It
            // is threaded separately from `last` because the `RateLimitInfo`
            // carried on the error is response-specific and so *not* sticky.
            let sticky_retry_after = match self.send_once(method, &url, cfg, body_bytes.as_deref())
            {
                Ok(mut raw) => {
                    let status = raw.status().as_u16();
                    // Two views of the same headers. `rate_limit` describes only
                    // *this* response and is what the caller is handed; `merged`
                    // is the sticky client-global state behind
                    // `Client::rate_limit()` and backoff.
                    let rate_limit = crate::ratelimit::parse_headers(raw.headers());
                    let merged = self.inner.limiter.merge(&rate_limit);
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
                    let api = ApiError {
                        status,
                        message,
                        rate_limit,
                        body: bytes,
                    };

                    if !is_transient(api.status, &api.message) && api.status != 429 {
                        return Err(Error::Api(api));
                    }
                    last = Some(Error::Api(api));
                    merged.retry_after
                }
                Err(e) => {
                    last = Some(Error::Transport(Box::new(e)));
                    // A transport failure produced no headers at all; Go falls
                    // back to computed backoff here rather than reusing an
                    // earlier response's `Retry-After`.
                    None
                }
            };

            if attempt < self.inner.max_attempts {
                std::thread::sleep(self.backoff(attempt, sticky_retry_after));
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
            assert!(
                !is_transient(status, "ordinary message"),
                "{status} should be fatal"
            );
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
    fn backoff_clamps_a_garbled_retry_after_instead_of_panicking() {
        // Every one of these makes `Duration::from_secs_f64` panic outright,
        // and all three are values `f64::from_str` accepts from a `Retry-After`
        // header: `"inf"`, `"1e400"` (overflows to infinity) and a finite value
        // larger than `u64::MAX` seconds.
        let c = Client::new().unwrap();
        for raw in [
            f64::INFINITY,
            "1e400".parse::<f64>().unwrap(),
            f64::MAX,
            1e30,
        ] {
            assert_eq!(
                c.backoff(1, Some(raw)),
                Duration::from_secs_f64(MAX_RETRY_AFTER_SECS),
                "Retry-After {raw} must clamp, not panic or sleep for ever"
            );
        }
    }

    #[test]
    fn backoff_honours_a_sane_retry_after_unchanged() {
        let c = Client::new().unwrap();
        assert_eq!(c.backoff(1, Some(2.5)), Duration::from_secs_f64(2.5));
        assert_eq!(
            c.backoff(1, None),
            Duration::from_secs(2),
            "with no Retry-After, backoff is attempt * wall_time * 2"
        );
    }

    #[test]
    fn transient_message_matching_ignores_case() {
        assert!(is_transient(400, "SOMETHING BAD HAS HAPPENED"));
        assert!(is_transient(400, "something bad has happened"));
    }
}
