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
    /// `api.rate_limit.retry_after` is sticky by design: `RateLimiter::update_from_headers`
    /// only overwrites fields a response actually sent, so once a 429 supplies
    /// `Retry-After`, a later attempt's error (e.g. a 503 with no such header)
    /// still carries it and this still sleeps for it. That looks like a bug but
    /// isn't one — `goensemblrest/ratelimit.go` behaves identically, so this is a
    /// faithful port, not a divergence to "fix".
    fn backoff(&self, attempt: u32, last: Option<&Error>) -> Duration {
        if let Some(Error::Api(api)) = last
            && let Some(retry_after) = api.rate_limit.retry_after
            && retry_after > 0.0
        {
            return Duration::from_secs_f64(retry_after);
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
    fn transient_message_matching_ignores_case() {
        assert!(is_transient(400, "SOMETHING BAD HAS HAPPENED"));
        assert!(is_transient(400, "something bad has happened"));
    }
}
