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
    // Consumed by request execution starting in Task 9; until then nothing
    // outside `#[cfg(test)]` calls it.
    #[allow(dead_code)]
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
    // Consumed by request execution starting in Task 9; until then nothing
    // outside `#[cfg(test)]` calls it.
    #[allow(dead_code)]
    pub(crate) fn update_from_headers(&self, headers: &HeaderMap) -> RateLimitInfo {
        let get_i64 =
            |name: &str| -> Option<i64> { headers.get(name)?.to_str().ok()?.trim().parse().ok() };
        let get_f64 =
            |name: &str| -> Option<f64> { headers.get(name)?.to_str().ok()?.trim().parse().ok() };

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
        assert!(
            start.elapsed() < Duration::from_millis(50),
            "should not have slept"
        );
    }

    #[test]
    fn blocks_once_the_window_is_full() {
        let rl = RateLimiter::new(2, Duration::from_millis(300));
        let start = Instant::now();
        for _ in 0..3 {
            rl.wait();
        }
        // The third call must wait for the first timestamp to age out.
        assert!(
            start.elapsed() >= Duration::from_millis(250),
            "elapsed {:?}",
            start.elapsed()
        );
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
        h.insert(
            "X-RateLimit-Reset",
            HeaderValue::from_static("not-a-number"),
        );
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
        assert_eq!(
            info.limit,
            Some(100),
            "previously seen values must be retained"
        );
    }
}
