//! Endpoint method definitions.
//!
//! Task 10 fills this module with the full endpoint table; for now it defines
//! only [`Method`] and a temporary [`Client::call_raw`] used by tests.

/// The HTTP method an endpoint uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// HTTP GET.
    Get,
    /// HTTP POST.
    Post,
}

impl crate::Client {
    /// Executes a raw request against an explicit path template.
    ///
    /// Prefer the typed endpoint methods; this exists for tests and for callers
    /// working with paths not yet in the endpoint table.
    pub fn call_raw(
        &self,
        method: Method,
        path_template: &str,
        path_params: &[(&str, &str)],
        body: Option<&serde_json::Value>,
        opts: &[crate::RequestOption<'_>],
    ) -> crate::Result<crate::Response> {
        let cfg = crate::options::resolve(crate::DEFAULT_CONTENT_TYPE, opts);
        self.execute(method, path_template, path_params, body, &cfg)
    }
}
