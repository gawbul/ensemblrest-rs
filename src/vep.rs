//! Variant Effect Predictor (VEP) consequences, by HGVS notation, identifier or region.

use serde_json::json;

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Fetches variant consequences based on an HGVS notation.
    pub fn get_variant_consequences_by_hgvs_notation(
        &self,
        species: &str,
        hgvs_notation: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getVariantConsequencesByHGVSNotation",
            &[("species", species), ("hgvs_notation", hgvs_notation)],
            None,
            opts,
        )
    }

    /// Fetches variant consequences for multiple HGVS notations.
    pub fn get_variant_consequences_by_multiple_hgvs_notations(
        &self,
        species: &str,
        hgvs_notations: &[&str],
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getVariantConsequencesByMultipleHGVSNotations",
            &[("species", species)],
            Some(&json!({ "hgvs_notations": hgvs_notations })),
            opts,
        )
    }

    /// Fetches variant consequences based on a variant identifier.
    pub fn get_variant_consequences_by_id(
        &self,
        species: &str,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getVariantConsequencesById",
            &[("species", species), ("id", id)],
            None,
            opts,
        )
    }

    /// Fetches variant consequences for multiple IDs.
    pub fn get_variant_consequences_by_multiple_ids(
        &self,
        species: &str,
        ids: &[&str],
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getVariantConsequencesByMultipleIds",
            &[("species", species)],
            Some(&json!({ "ids": ids })),
            opts,
        )
    }

    /// Fetches variant consequences for a given region and allele.
    pub fn get_variant_consequences_by_region(
        &self,
        species: &str,
        region: &str,
        allele: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getVariantConsequencesByRegion",
            &[("species", species), ("region", region), ("allele", allele)],
            None,
            opts,
        )
    }

    /// Fetches variant consequences for multiple regions.
    pub fn get_variant_consequences_by_multiple_regions(
        &self,
        species: &str,
        variants: &[&str],
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getVariantConsequencesByMultipleRegions",
            &[("species", species)],
            Some(&json!({ "variants": variants })),
            opts,
        )
    }
}
