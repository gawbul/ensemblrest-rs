//! GA4GH-compliant endpoints: Beacon, and search/lookup across GA4GH resources
//! (features, callsets, datasets, featuresets, variants, variantsets, references,
//! referencesets and variant annotation(sets)).

use serde_json::{Value, json};

use crate::options::{RequestOption, query};
use crate::{Client, Response, Result};

/// Builds a JSON object containing only the fields with a `Some` value.
///
/// The GA4GH search endpoints accept a body of optional filter fields (page
/// tokens, ids, coordinates, ...); this keeps a caller from having to send
/// `null` for every field it does not care about.
fn search_body(fields: &[(&str, Option<Value>)]) -> Value {
    let map: serde_json::Map<String, Value> = fields
        .iter()
        .filter_map(|(k, v)| v.clone().map(|v| ((*k).to_string(), v)))
        .collect();
    Value::Object(map)
}

impl Client {
    /// Returns Beacon metadata.
    pub fn get_ga4gh_beacon(&self, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getGA4GHBeacon", &[], None, opts)
    }

    /// Executes a Beacon query for allele information via GET.
    #[allow(clippy::too_many_arguments)]
    pub fn get_ga4gh_beacon_query(
        &self,
        alternate_bases: &str,
        assembly_id: &str,
        reference_bases: &str,
        reference_name: &str,
        start: i64,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let start = start.to_string();
        let mut all_opts = vec![
            query("alternateBases", alternate_bases),
            query("assemblyId", assembly_id),
            query("referenceBases", reference_bases),
            query("referenceName", reference_name),
            query("start", &start),
        ];
        all_opts.extend_from_slice(opts);
        self.call("getGA4GHBeaconQuery", &[], None, &all_opts)
    }

    /// Executes a Beacon query via POST.
    #[allow(clippy::too_many_arguments)]
    pub fn post_ga4gh_beacon_query(
        &self,
        alternate_bases: Option<&str>,
        assembly_id: Option<&str>,
        end: Option<i64>,
        reference_bases: Option<&str>,
        reference_name: Option<&str>,
        start: Option<i64>,
        variant_type: Option<&str>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("alternateBases", alternate_bases.map(|v| json!(v))),
            ("assemblyId", assembly_id.map(|v| json!(v))),
            ("end", end.map(|v| json!(v))),
            ("referenceBases", reference_bases.map(|v| json!(v))),
            ("referenceName", reference_name.map(|v| json!(v))),
            ("start", start.map(|v| json!(v))),
            ("variantType", variant_type.map(|v| json!(v))),
        ]);
        self.call("postGA4GHBeaconQuery", &[], Some(&body), opts)
    }

    /// Returns the GA4GH record for a specific sequence feature.
    pub fn get_ga4gh_features_by_id(
        &self,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getGA4GHFeaturesById", &[("id", id)], None, opts)
    }

    /// Searches for sequence annotation features in GA4GH format.
    pub fn search_ga4gh_features(
        &self,
        end: Option<i64>,
        reference_name: Option<&str>,
        start: Option<i64>,
        feature_set_id: Option<&str>,
        parent_id: Option<&str>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("end", end.map(|v| json!(v))),
            ("referenceName", reference_name.map(|v| json!(v))),
            ("start", start.map(|v| json!(v))),
            ("featureSetId", feature_set_id.map(|v| json!(v))),
            ("parentId", parent_id.map(|v| json!(v))),
        ]);
        self.call("searchGA4GHFeatures", &[], Some(&body), opts)
    }

    /// Returns sets of genotype calls for specific samples in GA4GH format.
    pub fn search_ga4gh_callset(
        &self,
        variant_set_id: Option<&str>,
        name: Option<&str>,
        page_token: Option<&str>,
        page_size: Option<i64>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("variantSetId", variant_set_id.map(|v| json!(v))),
            ("name", name.map(|v| json!(v))),
            ("pageToken", page_token.map(|v| json!(v))),
            ("pageSize", page_size.map(|v| json!(v))),
        ]);
        self.call("searchGA4GHCallset", &[], Some(&body), opts)
    }

    /// Returns the GA4GH record for a CallSet by ID.
    pub fn get_ga4gh_callset_by_id(
        &self,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getGA4GHCallsetById", &[("id", id)], None, opts)
    }

    /// Searches for datasets in GA4GH format.
    pub fn search_ga4gh_datasets(
        &self,
        page_token: Option<&str>,
        page_size: Option<i64>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("pageToken", page_token.map(|v| json!(v))),
            ("pageSize", page_size.map(|v| json!(v))),
        ]);
        self.call("searchGA4GHDatasets", &[], Some(&body), opts)
    }

    /// Returns a dataset in GA4GH format by ID.
    pub fn get_ga4gh_datasets_by_id(
        &self,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getGA4GHDatasetsById", &[("id", id)], None, opts)
    }

    /// Searches for feature sets in GA4GH format.
    pub fn search_ga4gh_featuresets(
        &self,
        dataset_id: Option<&str>,
        page_token: Option<&str>,
        page_size: Option<i64>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("datasetId", dataset_id.map(|v| json!(v))),
            ("pageToken", page_token.map(|v| json!(v))),
            ("pageSize", page_size.map(|v| json!(v))),
        ]);
        self.call("searchGA4GHFeaturesets", &[], Some(&body), opts)
    }

    /// Returns a feature set by ID.
    pub fn get_ga4gh_featuresets_by_id(
        &self,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getGA4GHFeaturesetsById", &[("id", id)], None, opts)
    }

    /// Returns a specific variant by ID.
    pub fn get_ga4gh_variants_by_id(
        &self,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getGA4GHVariantsById", &[("id", id)], None, opts)
    }

    /// Searches for variant annotations in GA4GH format.
    #[allow(clippy::too_many_arguments)]
    pub fn search_ga4gh_variant_annotations(
        &self,
        variant_annotation_set_id: Option<&str>,
        effects: Option<&[&str]>,
        end: Option<i64>,
        page_size: Option<i64>,
        page_token: Option<&str>,
        reference_id: Option<&str>,
        reference_name: Option<&str>,
        start: Option<i64>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            (
                "variantAnnotationSetId",
                variant_annotation_set_id.map(|v| json!(v)),
            ),
            ("effects", effects.map(|v| json!(v))),
            ("end", end.map(|v| json!(v))),
            ("pageSize", page_size.map(|v| json!(v))),
            ("pageToken", page_token.map(|v| json!(v))),
            ("referenceId", reference_id.map(|v| json!(v))),
            ("referenceName", reference_name.map(|v| json!(v))),
            ("start", start.map(|v| json!(v))),
        ]);
        self.call("searchGA4GHVariantAnnotations", &[], Some(&body), opts)
    }

    /// Searches for variant calls in GA4GH format.
    #[allow(clippy::too_many_arguments)]
    pub fn search_ga4gh_variants(
        &self,
        variant_set_id: Option<&str>,
        call_set_ids: Option<&[&str]>,
        reference_name: Option<&str>,
        start: Option<i64>,
        end: Option<i64>,
        page_token: Option<&str>,
        page_size: Option<i64>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("variantSetId", variant_set_id.map(|v| json!(v))),
            ("callSetIds", call_set_ids.map(|v| json!(v))),
            ("referenceName", reference_name.map(|v| json!(v))),
            ("start", start.map(|v| json!(v))),
            ("end", end.map(|v| json!(v))),
            ("pageToken", page_token.map(|v| json!(v))),
            ("pageSize", page_size.map(|v| json!(v))),
        ]);
        self.call("searchGA4GHVariants", &[], Some(&body), opts)
    }

    /// Searches for variant sets in GA4GH format.
    pub fn search_ga4gh_variantsets(
        &self,
        dataset_id: Option<&str>,
        page_token: Option<&str>,
        page_size: Option<i64>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("datasetId", dataset_id.map(|v| json!(v))),
            ("pageToken", page_token.map(|v| json!(v))),
            ("pageSize", page_size.map(|v| json!(v))),
        ]);
        self.call("searchGA4GHVariantsets", &[], Some(&body), opts)
    }

    /// Returns a variant set by ID.
    pub fn get_ga4gh_variantsets_by_id(
        &self,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getGA4GHVariantsetsById", &[("id", id)], None, opts)
    }

    /// Searches for reference sequences in GA4GH format.
    pub fn search_ga4gh_references(
        &self,
        reference_set_id: Option<&str>,
        md5checksum: Option<&str>,
        accession: Option<&str>,
        page_token: Option<&str>,
        page_size: Option<i64>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("referenceSetId", reference_set_id.map(|v| json!(v))),
            ("md5checksum", md5checksum.map(|v| json!(v))),
            ("accession", accession.map(|v| json!(v))),
            ("pageToken", page_token.map(|v| json!(v))),
            ("pageSize", page_size.map(|v| json!(v))),
        ]);
        self.call("searchGA4GHReferences", &[], Some(&body), opts)
    }

    /// Returns reference sequence data by ID.
    pub fn get_ga4gh_references_by_id(
        &self,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getGA4GHReferencesById", &[("id", id)], None, opts)
    }

    /// Searches for reference sets in GA4GH format.
    pub fn search_ga4gh_referencesets(
        &self,
        accession: Option<&str>,
        page_token: Option<&str>,
        page_size: Option<i64>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("accession", accession.map(|v| json!(v))),
            ("pageToken", page_token.map(|v| json!(v))),
            ("pageSize", page_size.map(|v| json!(v))),
        ]);
        self.call("searchGA4GHReferencesets", &[], Some(&body), opts)
    }

    /// Returns a reference set by ID.
    pub fn get_ga4gh_referencesets_by_id(
        &self,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getGA4GHReferencesetsById", &[("id", id)], None, opts)
    }

    /// Searches for annotation sets in GA4GH format.
    pub fn search_ga4gh_variant_annotationsets(
        &self,
        variant_set_id: Option<&str>,
        page_token: Option<&str>,
        page_size: Option<i64>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("variantSetId", variant_set_id.map(|v| json!(v))),
            ("pageToken", page_token.map(|v| json!(v))),
            ("pageSize", page_size.map(|v| json!(v))),
        ]);
        self.call("searchGA4GHVariantAnnotationsets", &[], Some(&body), opts)
    }

    /// Returns metadata for an annotation set by ID.
    pub fn get_ga4gh_variant_annotationsets_by_id(
        &self,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getGA4GHVariantAnnotationsetsById",
            &[("id", id)],
            None,
            opts,
        )
    }
}
