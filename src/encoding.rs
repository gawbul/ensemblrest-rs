//! Percent-encoding, path-template resolution and query-string building.
//!
//! Hand-written on `std` so the crate needs neither `percent-encoding`, `url`
//! nor `regex`.

use crate::error::{Error, Result};

#[allow(dead_code)]
const HEX: &[u8; 16] = b"0123456789ABCDEF";

#[allow(dead_code)]
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
#[allow(dead_code)]
const fn is_path_safe(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~' | b':')
}

/// Returns `true` for bytes that may appear literally in a form-encoded component.
///
/// This is the unreserved set only. Note `:` is *not* included: query strings
/// follow `application/x-www-form-urlencoded`, matching Go's `url.QueryEscape`.
#[allow(dead_code)]
const fn is_form_safe(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~')
}

/// Percent-encodes a URL path segment, preserving colons.
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
        assert_eq!(
            encode_path_segment("homo_sapiens:BRCA2"),
            "homo_sapiens:BRCA2"
        );
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
        assert!(
            matches!(&err, Error::MissingParam(n) if n == "id"),
            "got {err:?}"
        );

        let err = resolve_path("/lookup/id/{{id}}", &[("id", "")]).unwrap_err();
        assert!(
            matches!(&err, Error::MissingParam(n) if n == "id"),
            "got {err:?}"
        );
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
