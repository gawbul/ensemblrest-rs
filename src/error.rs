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
        200 => (
            "OK",
            "Request was a success. Only process data from the service when you receive this code",
        ),
        400 => (
            "Bad Request",
            "Occurs during exceptional circumstances such as the service is unable to find an ID. If JSON, the object contains the error message",
        ),
        403 => (
            "Forbidden",
            "You are submitting far too many requests and have been temporarily forbidden access. Wait and retry with a maximum of 15 requests per second",
        ),
        404 => (
            "Not Found",
            "Indicates a badly formatted request. Check your URL",
        ),
        408 => (
            "Timeout",
            "The request was not processed in time. Wait and retry later",
        ),
        415 => (
            "Unsupported Media Type",
            "The server is refusing to service the request because the entity format is not supported",
        ),
        429 => (
            "Too Many Requests",
            "You have been rate-limited; wait and retry",
        ),
        500 => (
            "Internal Server Error",
            "Internal server error. Check your input or contact the Ensembl team if issue persists",
        ),
        503 => (
            "Service Unavailable",
            "The service is temporarily down; retry after a pause",
        ),
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
        if self.status == 429
            && let Some(retry_after) = self.rate_limit.retry_after
        {
            return write!(
                f,
                "EnsEMBL REST API returned a {} ({name}): {msg} \
                 (Rate limit hit: Retry after {} seconds)",
                self.status, retry_after as i64
            );
        }
        write!(
            f,
            "EnsEMBL REST API returned a {} ({name}): {msg}",
            self.status
        )
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
                write!(
                    f,
                    "ensembl: maximum retry attempts reached: {last} (attempts: {attempts})"
                )
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
        let e = Error::MaxRetries {
            attempts: 5,
            last: Box::new(inner),
        };
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
