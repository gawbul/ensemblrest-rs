//! Features overlapping a region, identifier or translation.

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Retrieves features that overlap a region defined by the given identifier.
    pub fn get_overlap_by_id(&self, id: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getOverlapById", &[("id", id)], None, opts)
    }

    /// Retrieves features that overlap a given region.
    pub fn get_overlap_by_region(
        &self,
        species: &str,
        region: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getOverlapByRegion",
            &[("species", species), ("region", region)],
            None,
            opts,
        )
    }

    /// Retrieves features related to a specific Translation (e.g. domains, variants).
    pub fn get_overlap_by_translation(
        &self,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getOverlapByTranslation", &[("id", id)], None, opts)
    }
}
