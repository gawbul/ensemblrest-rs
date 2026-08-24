//! GA4GH-compliant endpoints: Beacon, and search/lookup across GA4GH resources
//! (features, callsets, datasets, featuresets, variants, variantsets, references,
//! referencesets and variant annotation(sets)).
//!
//! # Query structs
//!
//! Every GA4GH endpoint that takes more than two body fields takes a
//! `Ga4gh*Query` struct instead of a row of positional `Option`s. Those bodies
//! are made almost entirely of adjacent same-typed optional filters -- three
//! consecutive `Option<&str>`s on `/ga4gh/references/search`, a
//! `referenceId`/`referenceName` pair on `/ga4gh/variantannotations/search` --
//! so a transposed pair of arguments would compile, run, and silently query the
//! wrong thing. Naming the fields removes that failure mode. Fields left `None`
//! are omitted from the request body entirely, so `..Default::default()` sends
//! nothing for what you did not set. See the crate-level docs for an example.

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

/// The POST body for [`Client::post_ga4gh_beacon_query`].
///
/// `alternate_bases` and `reference_bases` are adjacent `Option<&str>`s that a
/// positional signature would let you swap without complaint.
#[derive(Debug, Default, Clone)]
pub struct Ga4ghBeaconQuery<'a> {
    /// `alternateBases`: the alternate (variant) allele.
    pub alternate_bases: Option<&'a str>,
    /// `assemblyId`: the assembly the coordinates refer to, e.g. `GRCh38`.
    pub assembly_id: Option<&'a str>,
    /// `end`: the end coordinate of the region of interest.
    pub end: Option<i64>,
    /// `referenceBases`: the reference allele at `start`.
    pub reference_bases: Option<&'a str>,
    /// `referenceName`: the reference sequence (chromosome) name.
    pub reference_name: Option<&'a str>,
    /// `start`: the start coordinate of the region of interest.
    pub start: Option<i64>,
    /// `variantType`: the type of variant being queried, e.g. `DEL`.
    pub variant_type: Option<&'a str>,
}

/// The POST body for [`Client::search_ga4gh_features`].
#[derive(Debug, Default, Clone)]
pub struct Ga4ghFeaturesQuery<'a> {
    /// `end`: the end coordinate of the region to search.
    pub end: Option<i64>,
    /// `referenceName`: the reference sequence (chromosome) name.
    pub reference_name: Option<&'a str>,
    /// `start`: the start coordinate of the region to search.
    pub start: Option<i64>,
    /// `featureSetId`: restrict the search to one feature set.
    pub feature_set_id: Option<&'a str>,
    /// `parentId`: restrict the search to children of this feature.
    pub parent_id: Option<&'a str>,
}

/// The POST body for [`Client::search_ga4gh_callset`].
#[derive(Debug, Default, Clone)]
pub struct Ga4ghCallsetQuery<'a> {
    /// `variantSetId`: the variant set whose call sets are wanted.
    pub variant_set_id: Option<&'a str>,
    /// `name`: restrict the search to call sets with this name.
    pub name: Option<&'a str>,
    /// `pageToken`: the continuation token from a previous page.
    ///
    /// Passed through verbatim as a [`Value`] because Ensembl's documentation
    /// (`Integer`) and the GA4GH specification (`string`) disagree on its type.
    pub page_token: Option<&'a Value>,
    /// `pageSize`: the maximum number of results per page.
    pub page_size: Option<i64>,
}

/// The POST body for [`Client::search_ga4gh_featuresets`].
#[derive(Debug, Default, Clone)]
pub struct Ga4ghFeaturesetsQuery<'a> {
    /// `datasetId`: the dataset whose feature sets are wanted.
    pub dataset_id: Option<&'a str>,
    /// `pageToken`: the continuation token from a previous page.
    ///
    /// Passed through verbatim as a [`Value`] because Ensembl's documentation
    /// (`Integer`) and the GA4GH specification (`string`) disagree on its type.
    pub page_token: Option<&'a Value>,
    /// `pageSize`: the maximum number of results per page.
    pub page_size: Option<i64>,
}

/// The POST body for [`Client::search_ga4gh_variant_annotations`].
///
/// `reference_id` and `reference_name` are adjacent `Option<&str>`s that a
/// positional signature would let you swap without complaint.
#[derive(Debug, Default, Clone)]
pub struct Ga4ghVariantAnnotationsQuery<'a> {
    /// `variantAnnotationSetId`: the annotation set to search.
    pub variant_annotation_set_id: Option<&'a str>,
    /// `effects`: restrict results to these consequence terms.
    ///
    /// Passed through verbatim as a [`Value`] because Ensembl documents this as
    /// an array of `OntologyTerm` *objects*, not bare strings; `&[&str]` could
    /// not express a valid request.
    pub effects: Option<&'a Value>,
    /// `end`: the end coordinate of the region to search.
    pub end: Option<i64>,
    /// `pageSize`: the maximum number of results per page.
    pub page_size: Option<i64>,
    /// `pageToken`: the continuation token from a previous page.
    ///
    /// Passed through verbatim as a [`Value`] because Ensembl's documentation
    /// (`Integer`) and the GA4GH specification (`string`) disagree on its type.
    pub page_token: Option<&'a Value>,
    /// `referenceId`: the reference sequence *identifier* to search.
    pub reference_id: Option<&'a str>,
    /// `referenceName`: the reference sequence (chromosome) *name* to search.
    pub reference_name: Option<&'a str>,
    /// `start`: the start coordinate of the region to search.
    pub start: Option<i64>,
}

/// The POST body for [`Client::search_ga4gh_variants`].
#[derive(Debug, Default, Clone)]
pub struct Ga4ghVariantsQuery<'a> {
    /// `variantSetId`: the variant set to search.
    pub variant_set_id: Option<&'a str>,
    /// `callSetIds`: restrict results to calls from these call sets.
    pub call_set_ids: Option<&'a [&'a str]>,
    /// `referenceName`: the reference sequence (chromosome) name.
    pub reference_name: Option<&'a str>,
    /// `start`: the start coordinate of the region to search.
    pub start: Option<i64>,
    /// `end`: the end coordinate of the region to search.
    pub end: Option<i64>,
    /// `pageToken`: the continuation token from a previous page.
    ///
    /// Passed through verbatim as a [`Value`] because Ensembl's documentation
    /// (`Integer`) and the GA4GH specification (`string`) disagree on its type.
    pub page_token: Option<&'a Value>,
    /// `pageSize`: the maximum number of results per page.
    pub page_size: Option<i64>,
}

/// The POST body for [`Client::search_ga4gh_variantsets`].
#[derive(Debug, Default, Clone)]
pub struct Ga4ghVariantsetsQuery<'a> {
    /// `datasetId`: the dataset whose variant sets are wanted.
    pub dataset_id: Option<&'a str>,
    /// `pageToken`: the continuation token from a previous page.
    ///
    /// Passed through verbatim as a [`Value`] because Ensembl's documentation
    /// (`Integer`) and the GA4GH specification (`string`) disagree on its type.
    pub page_token: Option<&'a Value>,
    /// `pageSize`: the maximum number of results per page.
    pub page_size: Option<i64>,
}

/// The POST body for [`Client::search_ga4gh_references`].
///
/// `reference_set_id`, `md5checksum` and `accession` are three consecutive
/// `Option<&str>`s -- the worst transposition hazard in this module, and the
/// reason this endpoint takes a struct.
#[derive(Debug, Default, Clone)]
pub struct Ga4ghReferencesQuery<'a> {
    /// `referenceSetId`: the reference set whose references are wanted.
    pub reference_set_id: Option<&'a str>,
    /// `md5checksum`: match the reference sequence with this MD5 checksum.
    pub md5checksum: Option<&'a str>,
    /// `accession`: match the reference sequence with this accession.
    pub accession: Option<&'a str>,
    /// `pageToken`: the continuation token from a previous page.
    ///
    /// Passed through verbatim as a [`Value`] because Ensembl's documentation
    /// (`Integer`) and the GA4GH specification (`string`) disagree on its type.
    pub page_token: Option<&'a Value>,
    /// `pageSize`: the maximum number of results per page.
    pub page_size: Option<i64>,
}

/// The POST body for [`Client::search_ga4gh_referencesets`].
#[derive(Debug, Default, Clone)]
pub struct Ga4ghReferencesetsQuery<'a> {
    /// `accession`: match the reference set with this accession.
    pub accession: Option<&'a str>,
    /// `pageToken`: the continuation token from a previous page.
    ///
    /// Passed through verbatim as a [`Value`] because Ensembl's documentation
    /// (`Integer`) and the GA4GH specification (`string`) disagree on its type.
    pub page_token: Option<&'a Value>,
    /// `pageSize`: the maximum number of results per page.
    pub page_size: Option<i64>,
}

/// The POST body for [`Client::search_ga4gh_variant_annotationsets`].
#[derive(Debug, Default, Clone)]
pub struct Ga4ghVariantAnnotationsetsQuery<'a> {
    /// `variantSetId`: the variant set whose annotation sets are wanted.
    pub variant_set_id: Option<&'a str>,
    /// `pageToken`: the continuation token from a previous page.
    ///
    /// Passed through verbatim as a [`Value`] because Ensembl's documentation
    /// (`Integer`) and the GA4GH specification (`string`) disagree on its type.
    pub page_token: Option<&'a Value>,
    /// `pageSize`: the maximum number of results per page.
    pub page_size: Option<i64>,
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
    pub fn post_ga4gh_beacon_query(
        &self,
        query: &Ga4ghBeaconQuery<'_>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("alternateBases", query.alternate_bases.map(|v| json!(v))),
            ("assemblyId", query.assembly_id.map(|v| json!(v))),
            ("end", query.end.map(|v| json!(v))),
            ("referenceBases", query.reference_bases.map(|v| json!(v))),
            ("referenceName", query.reference_name.map(|v| json!(v))),
            ("start", query.start.map(|v| json!(v))),
            ("variantType", query.variant_type.map(|v| json!(v))),
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
        query: &Ga4ghFeaturesQuery<'_>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("end", query.end.map(|v| json!(v))),
            ("referenceName", query.reference_name.map(|v| json!(v))),
            ("start", query.start.map(|v| json!(v))),
            ("featureSetId", query.feature_set_id.map(|v| json!(v))),
            ("parentId", query.parent_id.map(|v| json!(v))),
        ]);
        self.call("searchGA4GHFeatures", &[], Some(&body), opts)
    }

    /// Returns sets of genotype calls for specific samples in GA4GH format.
    pub fn search_ga4gh_callset(
        &self,
        query: &Ga4ghCallsetQuery<'_>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("variantSetId", query.variant_set_id.map(|v| json!(v))),
            ("name", query.name.map(|v| json!(v))),
            ("pageToken", query.page_token.cloned()),
            ("pageSize", query.page_size.map(|v| json!(v))),
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
    ///
    /// `page_token`'s type is ambiguous between sources (Ensembl docs say
    /// `Integer`, the GA4GH spec says `string`), so it is passed through
    /// verbatim; see [`Client::call`] for full control.
    ///
    /// This endpoint keeps positional parameters rather than a `Ga4gh*Query`
    /// struct: with only two body fields, of different types, there is no
    /// transposition to guard against.
    pub fn search_ga4gh_datasets(
        &self,
        page_token: Option<&Value>,
        page_size: Option<i64>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("pageToken", page_token.cloned()),
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
        query: &Ga4ghFeaturesetsQuery<'_>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("datasetId", query.dataset_id.map(|v| json!(v))),
            ("pageToken", query.page_token.cloned()),
            ("pageSize", query.page_size.map(|v| json!(v))),
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
    pub fn search_ga4gh_variant_annotations(
        &self,
        query: &Ga4ghVariantAnnotationsQuery<'_>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            (
                "variantAnnotationSetId",
                query.variant_annotation_set_id.map(|v| json!(v)),
            ),
            ("effects", query.effects.cloned()),
            ("end", query.end.map(|v| json!(v))),
            ("pageSize", query.page_size.map(|v| json!(v))),
            ("pageToken", query.page_token.cloned()),
            ("referenceId", query.reference_id.map(|v| json!(v))),
            ("referenceName", query.reference_name.map(|v| json!(v))),
            ("start", query.start.map(|v| json!(v))),
        ]);
        self.call("searchGA4GHVariantAnnotations", &[], Some(&body), opts)
    }

    /// Searches for variant calls in GA4GH format.
    pub fn search_ga4gh_variants(
        &self,
        query: &Ga4ghVariantsQuery<'_>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("variantSetId", query.variant_set_id.map(|v| json!(v))),
            ("callSetIds", query.call_set_ids.map(|v| json!(v))),
            ("referenceName", query.reference_name.map(|v| json!(v))),
            ("start", query.start.map(|v| json!(v))),
            ("end", query.end.map(|v| json!(v))),
            ("pageToken", query.page_token.cloned()),
            ("pageSize", query.page_size.map(|v| json!(v))),
        ]);
        self.call("searchGA4GHVariants", &[], Some(&body), opts)
    }

    /// Searches for variant sets in GA4GH format.
    pub fn search_ga4gh_variantsets(
        &self,
        query: &Ga4ghVariantsetsQuery<'_>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("datasetId", query.dataset_id.map(|v| json!(v))),
            ("pageToken", query.page_token.cloned()),
            ("pageSize", query.page_size.map(|v| json!(v))),
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
        query: &Ga4ghReferencesQuery<'_>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("referenceSetId", query.reference_set_id.map(|v| json!(v))),
            ("md5checksum", query.md5checksum.map(|v| json!(v))),
            ("accession", query.accession.map(|v| json!(v))),
            ("pageToken", query.page_token.cloned()),
            ("pageSize", query.page_size.map(|v| json!(v))),
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
        query: &Ga4ghReferencesetsQuery<'_>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("accession", query.accession.map(|v| json!(v))),
            ("pageToken", query.page_token.cloned()),
            ("pageSize", query.page_size.map(|v| json!(v))),
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
        query: &Ga4ghVariantAnnotationsetsQuery<'_>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let body = search_body(&[
            ("variantSetId", query.variant_set_id.map(|v| json!(v))),
            ("pageToken", query.page_token.cloned()),
            ("pageSize", query.page_size.map(|v| json!(v))),
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
