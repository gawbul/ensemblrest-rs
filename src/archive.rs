//! Archive lookups: latest version for stable identifiers.

use serde_json::json;

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Uses the given identifier to return its latest version.
    pub fn get_archive_by_id(&self, id: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getArchiveById", &[("id", id)], None, opts)
    }

    /// Retrieves the latest version for a set of identifiers.
    pub fn get_archive_by_multiple_ids(
        &self,
        ids: &[&str],
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getArchiveByMultipleIds",
            &[],
            Some(&json!({ "id": ids })),
            opts,
        )
    }
}
