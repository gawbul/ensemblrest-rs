//! Known variants (variation) and their recoded identifiers.

use serde_json::json;

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Translates a variant identifier, HGVS notation, or genomic SPDI notation to all possible variant IDs.
    pub fn get_variation_recoder_by_id(
        &self,
        species: &str,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getVariationRecoderById",
            &[("species", species), ("id", id)],
            None,
            opts,
        )
    }

    /// Translates a list of variant identifiers, HGVS notations, or SPDI notations.
    pub fn get_variation_recoder_by_multiple_ids(
        &self,
        species: &str,
        ids: &[&str],
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getVariationRecoderByMultipleIds",
            &[("species", species)],
            Some(&json!({ "ids": ids })),
            opts,
        )
    }

    /// Uses a variant identifier (e.g. rsID) to return variation features.
    pub fn get_variation_by_id(
        &self,
        species: &str,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getVariationById",
            &[("species", species), ("id", id)],
            None,
            opts,
        )
    }

    /// Returns variation features associated with a PMCID.
    pub fn get_variation_by_pmcid(
        &self,
        species: &str,
        pmcid: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getVariationByPMCID",
            &[("species", species), ("pmcid", pmcid)],
            None,
            opts,
        )
    }

    /// Returns variation features associated with a PubMed ID.
    pub fn get_variation_by_pmid(
        &self,
        species: &str,
        pmid: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getVariationByPMID",
            &[("species", species), ("pmid", pmid)],
            None,
            opts,
        )
    }

    /// Uses a list of variant identifiers to return variation features.
    pub fn get_variation_by_multiple_ids(
        &self,
        species: &str,
        ids: &[&str],
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getVariationByMultipleIds",
            &[("species", species)],
            Some(&json!({ "ids": ids })),
            opts,
        )
    }
}
