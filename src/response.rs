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
/// ```ignore
/// # use ensemblrest::{Client, types::LookupRecord};
/// # fn main() -> ensemblrest::Result<()> {
/// let client = Client::new()?;
/// let record: LookupRecord = client.get_lookup_by_id("ENSG00000157764", &[])?.json()?;
/// # Ok(())
/// # }
/// // TODO(Task 16): restore to no_run once Client and types exist
/// ```
#[derive(Debug, Clone)]
pub struct Response {
    status: u16,
    content_type: Option<String>,
    rate_limit: RateLimitInfo,
    body: Vec<u8>,
}

impl Response {
    // Consumed by request execution starting in Task 9; until then nothing
    // outside `#[cfg(test)]` calls it.
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) fn new(
        status: u16,
        content_type: Option<String>,
        rate_limit: RateLimitInfo,
        body: Vec<u8>,
    ) -> Self {
        Self {
            status,
            content_type,
            rate_limit,
            body,
        }
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
        assert_eq!(
            resp(br#"{"ping":1}"#).json::<Ping>().unwrap(),
            Ping { ping: 1 }
        );
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
        let rate = RateLimitInfo {
            remaining: Some(7),
            ..Default::default()
        };
        let r = Response::new(404, Some("text/plain".into()), rate, Vec::new());
        assert_eq!(r.status(), 404);
        assert_eq!(r.content_type(), Some("text/plain"));
        assert_eq!(r.rate_limit().remaining, Some(7));
    }
}
