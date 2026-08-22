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
// Consumed by request execution starting in Task 9; until then nothing
// outside `#[cfg(test)]` constructs or reads it.
#[cfg_attr(not(test), expect(dead_code))]
pub(crate) struct RequestConfig<'a> {
    pub(crate) query: Vec<(&'a str, &'a str)>,
    pub(crate) headers: Vec<(&'a str, &'a str)>,
    pub(crate) content_type: &'a str,
}

/// Folds `opts` over the endpoint's default content type.
///
/// The effective content type is the last [`RequestOption::ContentType`] given,
/// or `default_content_type` when none is.
// Consumed by endpoint dispatch starting in Task 10; until then nothing
// outside `#[cfg(test)]` calls it.
#[cfg_attr(not(test), expect(dead_code))]
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
        assert_eq!(
            cfg.query,
            vec![("feature", "gene"), ("feature", "transcript")]
        );
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
        assert_eq!(
            resolve("application/json", &opts).content_type,
            "text/x-fasta"
        );
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
