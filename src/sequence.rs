//! Sequence retrieval by identifier or genomic region.

use serde_json::json;

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Requests multiple types of sequence by stable identifier.
    pub fn get_sequence_by_id(&self, id: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getSequenceById", &[("id", id)], None, opts)
    }

    /// Requests multiple types of sequence by a stable identifier list.
    pub fn get_sequence_by_multiple_ids(
        &self,
        ids: &[&str],
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getSequenceByMultipleIds",
            &[],
            Some(&json!({ "ids": ids })),
            opts,
        )
    }

    /// Returns the genomic sequence of the specified region of the given species.
    pub fn get_sequence_by_region(
        &self,
        species: &str,
        region: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getSequenceByRegion",
            &[("species", species), ("region", region)],
            None,
            opts,
        )
    }

    /// Requests multiple types of sequence by a list of regions.
    pub fn get_sequence_by_multiple_regions(
        &self,
        species: &str,
        regions: &[&str],
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getSequenceByMultipleRegions",
            &[("species", species)],
            Some(&json!({ "regions": regions })),
            opts,
        )
    }
}
