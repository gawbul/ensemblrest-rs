//! Deserialization models for common Ensembl REST API responses.
//!
//! These cover the shapes the Go port models. They are a convenience, not a
//! requirement: [`crate::Response::json`] accepts any [`serde::Deserialize`]
//! type, including [`serde_json::Value`], so endpoints without a model here are
//! still fully usable.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Deserializes a JSON `null` as the field's `Default`, matching the null
/// tolerance the Go port gets for free from `encoding/json`.
///
/// Go's `encoding/json` silently decodes a JSON `null` into a `string` as
/// `""`, into numeric types as `0`, and into a slice as `nil`. Serde does
/// not: decoding `null` into a non-`Option` field such as `String` or `i64`
/// is a hard type error that fails deserialization of the *entire*
/// response, not just that field. These models were ported field-for-field
/// from the Go port's `types.go` on the assumption that its field types
/// carried over — but Go's null tolerance did not come with them. Ensembl's
/// API does send explicit `null` for fields it has no value for: for
/// example, `/info/species` currently returns `"strain": null` for most
/// species (including `homo_sapiens` itself) and `"accession": null` for
/// some.
///
/// `#[serde(default)]` on the struct does not fix this by itself: it only
/// supplies a default when a key is *missing* from the object, not when the
/// key is present with a `null` value. Pairing `#[serde(default,
/// deserialize_with = "null_to_default")]` on a field restores Go's
/// tolerance by treating a present-but-null value the same as an absent
/// one.
///
/// `Option<T>` fields are deliberately left without this attribute: they
/// already represent "may be absent or null" on their own, and wrapping
/// them here would be redundant. Bare [`Value`] fields don't need it
/// either: `Value`'s own `Deserialize` impl already accepts `null` (as
/// [`Value::Null`]) without error.
fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

/// The response from `/info/ping`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PingResponse {
    /// `1` when the service is up.
    #[serde(default, deserialize_with = "null_to_default")]
    pub ping: i64,
}

/// An archived identifier entry from the `/archive` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ArchiveRecord {
    /// The queried identifier.
    #[serde(default, deserialize_with = "null_to_default")]
    pub id: String,
    /// The latest versioned identifier.
    #[serde(default, deserialize_with = "null_to_default")]
    pub latest: String,
    /// The identifier version.
    #[serde(default, deserialize_with = "null_to_default")]
    pub version: i64,
    /// The Ensembl release this record came from.
    #[serde(default, deserialize_with = "null_to_default")]
    pub release: String,
    /// The assembly name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub assembly: String,
    /// The peptide sequence, when the identifier is a translation.
    pub peptide: Option<String>,
    /// The feature type.
    #[serde(rename = "type", default, deserialize_with = "null_to_default")]
    pub kind: String,
    /// Identifiers that may replace a retired one.
    #[serde(default, deserialize_with = "null_to_default")]
    pub possible_replacement: Vec<String>,
    /// Whether this is the current version.
    #[serde(default, deserialize_with = "null_to_default")]
    pub is_current: String,
}

/// A genomic feature returned by the `/lookup` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LookupRecord {
    /// The stable identifier.
    #[serde(default, deserialize_with = "null_to_default")]
    pub id: String,
    /// The feature type, for example `"Gene"` or `"Transcript"`.
    #[serde(default, deserialize_with = "null_to_default")]
    pub object_type: String,
    /// The species name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub species: String,
    /// The display label.
    #[serde(default, deserialize_with = "null_to_default")]
    pub display_name: String,
    /// A free-text description.
    #[serde(default, deserialize_with = "null_to_default")]
    pub description: String,
    /// The biotype, for example `"protein_coding"`.
    #[serde(default, deserialize_with = "null_to_default")]
    pub biotype: String,
    /// The sequence region, typically a chromosome name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub seq_region_name: String,
    /// The start coordinate.
    #[serde(default, deserialize_with = "null_to_default")]
    pub start: i64,
    /// The end coordinate.
    #[serde(default, deserialize_with = "null_to_default")]
    pub end: i64,
    /// The strand, `1` or `-1`.
    #[serde(default, deserialize_with = "null_to_default")]
    pub strand: i64,
    /// The annotation source.
    #[serde(default, deserialize_with = "null_to_default")]
    pub source: String,
    /// The feature version.
    #[serde(default, deserialize_with = "null_to_default")]
    pub version: i64,
    /// The database type.
    #[serde(default, deserialize_with = "null_to_default")]
    pub db_type: String,
    /// The assembly name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub assembly_name: String,
    /// The canonical transcript identifier.
    #[serde(default, deserialize_with = "null_to_default")]
    pub canonical_transcript: String,
    /// Child transcripts, populated by `?expand=1`.
    #[serde(rename = "Transcript", default, deserialize_with = "null_to_default")]
    pub transcripts: Vec<LookupRecord>,
    /// The translation, populated by `?expand=1`.
    #[serde(rename = "Translation")]
    pub translation: Option<Box<LookupRecord>>,
    /// Child exons, populated by `?expand=1`.
    #[serde(rename = "Exon", default, deserialize_with = "null_to_default")]
    pub exons: Vec<LookupRecord>,
    /// Any additional fields the API returned.
    ///
    /// The Go port declares an equivalent field as `json:"-"`, which means it can
    /// never be populated. Flattening captures what `?expand=1` actually returns.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// A nucleotide or protein sequence from the `/sequence` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SequenceRecord {
    /// The stable identifier the sequence was fetched for.
    #[serde(default, deserialize_with = "null_to_default")]
    pub id: String,
    /// The sequence itself.
    #[serde(default, deserialize_with = "null_to_default")]
    pub seq: String,
    /// A free-text description.
    #[serde(default, deserialize_with = "null_to_default")]
    pub desc: String,
    /// The original query string, for region-based lookups.
    #[serde(default, deserialize_with = "null_to_default")]
    pub query: String,
    /// The sequence version.
    #[serde(default, deserialize_with = "null_to_default")]
    pub version: i64,
    /// The molecule type, for example `"dna"` or `"protein"`.
    #[serde(default, deserialize_with = "null_to_default")]
    pub molecule: String,
}

/// Species metadata from `/info/species`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SpeciesRecord {
    /// The production name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub name: String,
    /// The human-readable name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub display_name: String,
    /// The NCBI taxonomy identifier.
    #[serde(default, deserialize_with = "null_to_default")]
    pub taxon_id: String,
    /// The species name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub species: String,
    /// The common name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub common_name: String,
    /// The Ensembl division.
    #[serde(default, deserialize_with = "null_to_default")]
    pub division: String,
    /// The Ensembl release.
    #[serde(default, deserialize_with = "null_to_default")]
    pub release: i64,
    /// The assembly name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub assembly: String,
    /// The assembly accession.
    #[serde(default, deserialize_with = "null_to_default")]
    pub accession: String,
    /// Alternative names.
    #[serde(default, deserialize_with = "null_to_default")]
    pub aliases: Vec<String>,
    /// Group memberships.
    #[serde(default, deserialize_with = "null_to_default")]
    pub groups: Vec<String>,
    /// The strain, where applicable.
    #[serde(default, deserialize_with = "null_to_default")]
    pub strain: String,
    /// The strain type.
    #[serde(default, deserialize_with = "null_to_default")]
    pub strain_type: String,
    /// Whether this is the reference genome for its group.
    #[serde(default, deserialize_with = "null_to_default")]
    pub is_reference: i64,
    /// Whether pan-taxonomic comparative data is available.
    #[serde(default, deserialize_with = "null_to_default")]
    pub has_pan_compara: i64,
    /// Whether variation data is available.
    #[serde(default, deserialize_with = "null_to_default")]
    pub has_variations: i64,
    /// Whether peptide comparative data is available.
    #[serde(default, deserialize_with = "null_to_default")]
    pub has_peptide_compara: i64,
    /// Whether whole-genome alignments are available.
    #[serde(default, deserialize_with = "null_to_default")]
    pub has_genome_alignments: i64,
    /// Whether synteny data is available.
    #[serde(default, deserialize_with = "null_to_default")]
    pub has_synteny: i64,
}

/// The wrapper `/info/species` returns.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SpeciesResponse {
    /// The species records.
    #[serde(default, deserialize_with = "null_to_default")]
    pub species: Vec<SpeciesRecord>,
}

/// Assembly information from `/info/assembly`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AssemblyInfo {
    /// The date the assembly was produced.
    #[serde(default, deserialize_with = "null_to_default")]
    pub assembly_date: String,
    /// The assembly level, for example `"chromosome"`.
    #[serde(default, deserialize_with = "null_to_default")]
    pub assembly_level: String,
    /// The assembly's INSDC accession.
    #[serde(default, deserialize_with = "null_to_default")]
    pub assembly_accession: String,
    /// The assembly name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub assembly_name: String,
    /// The default coordinate system version.
    #[serde(default, deserialize_with = "null_to_default")]
    pub default_coord_system_version: String,
    /// The date the gene build was produced.
    #[serde(default, deserialize_with = "null_to_default")]
    pub genebuild_date: String,
    /// The gene build method.
    #[serde(default, deserialize_with = "null_to_default")]
    pub genebuild_method: String,
    /// The date the gene build started.
    #[serde(default, deserialize_with = "null_to_default")]
    pub genebuild_start_date: String,
    /// The gene build version.
    #[serde(default, deserialize_with = "null_to_default")]
    pub genebuild_version: String,
    /// The names of the top-level sequence regions.
    #[serde(default, deserialize_with = "null_to_default")]
    pub top_level_region: Vec<String>,
    /// The karyotype, as an ordered list of chromosome names.
    #[serde(default, deserialize_with = "null_to_default")]
    pub karyotype: Vec<String>,
    /// The coordinate system versions available for this assembly.
    #[serde(default, deserialize_with = "null_to_default")]
    pub coord_system_versions: Vec<Value>,
}

/// Region metadata from `/info/assembly/:species/:region_name`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AssemblyRegionInfo {
    /// The region's length, in base pairs.
    #[serde(default, deserialize_with = "null_to_default")]
    pub length: i64,
    /// The coordinate system this region belongs to.
    #[serde(default, deserialize_with = "null_to_default")]
    pub coord_system: String,
    /// `1` when the region is a chromosome.
    #[serde(default, deserialize_with = "null_to_default")]
    pub is_chromosome: i64,
    /// `1` when the region is circular.
    #[serde(default, deserialize_with = "null_to_default")]
    pub is_circular: i64,
    /// The assembly name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub assembly: String,
    /// The sequence region name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub seq_region_name: String,
}

/// One homologous relationship between two genes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HomologyRecord {
    /// The homology type, for example `"ortholog_one2one"`.
    #[serde(rename = "type", default, deserialize_with = "null_to_default")]
    pub kind: String,
    /// The comparative analysis method used to call the homology.
    #[serde(default, deserialize_with = "null_to_default")]
    pub method_link_type: String,
    /// The taxonomic level the homology was called at.
    #[serde(default, deserialize_with = "null_to_default")]
    pub taxonomy_level: String,
    /// The source gene of the homology.
    pub source: Value,
    /// The target (homologous) gene.
    pub target: Value,
    /// The rate of non-synonymous substitutions.
    pub dn: Option<f64>,
    /// The rate of synonymous substitutions.
    pub ds: Option<f64>,
}

/// One queried gene's homology results, and its position in `HomologyResponse::data`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HomologyData {
    /// The queried gene's stable identifier.
    #[serde(default, deserialize_with = "null_to_default")]
    pub id: String,
    /// The homologous genes found.
    #[serde(default, deserialize_with = "null_to_default")]
    pub homologies: Vec<HomologyRecord>,
    /// The last common ancestor taxon, when known.
    #[serde(default, deserialize_with = "null_to_default")]
    pub last_common_ancestor: String,
}

/// The response from the `/homology` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HomologyResponse {
    /// One entry per queried gene.
    #[serde(default, deserialize_with = "null_to_default")]
    pub data: Vec<HomologyData>,
}

/// An external reference from the `/xrefs` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct XrefRecord {
    /// The external identifier.
    #[serde(default, deserialize_with = "null_to_default")]
    pub id: String,
    /// The external database name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub dbname: String,
    /// The display identifier.
    #[serde(default, deserialize_with = "null_to_default")]
    pub display_id: String,
    /// The primary identifier.
    #[serde(default, deserialize_with = "null_to_default")]
    pub primary_id: String,
    /// A free-text description.
    #[serde(default, deserialize_with = "null_to_default")]
    pub description: String,
    /// Known synonyms.
    #[serde(default, deserialize_with = "null_to_default")]
    pub synonyms: Vec<String>,
    /// The type of cross-reference information.
    #[serde(default, deserialize_with = "null_to_default")]
    pub info_type: String,
    /// Free-text information about the cross-reference.
    #[serde(default, deserialize_with = "null_to_default")]
    pub info_text: String,
}

/// A genetic variant from the `/variation` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct VariationRecord {
    /// The variant name, for example `"rs699"`.
    #[serde(default, deserialize_with = "null_to_default")]
    pub name: String,
    /// The source database.
    #[serde(default, deserialize_with = "null_to_default")]
    pub source: String,
    /// The variant class.
    #[serde(default, deserialize_with = "null_to_default")]
    pub var_class: String,
    /// The most severe consequence term.
    #[serde(default, deserialize_with = "null_to_default")]
    pub most_severe_consequence: String,
    /// The minor allele frequency.
    #[serde(rename = "MAF")]
    pub maf: Option<f64>,
    /// The minor allele.
    #[serde(default, deserialize_with = "null_to_default")]
    pub minor_allele: String,
    /// Supporting evidence terms.
    #[serde(default, deserialize_with = "null_to_default")]
    pub evidence: Vec<String>,
    /// Known synonyms.
    #[serde(default, deserialize_with = "null_to_default")]
    pub synonyms: Vec<String>,
    /// Clinical significance terms.
    #[serde(default, deserialize_with = "null_to_default")]
    pub clinical_significance: Vec<String>,
    /// Genomic mappings.
    #[serde(default, deserialize_with = "null_to_default")]
    pub mappings: Vec<Value>,
    /// Genotype records.
    #[serde(default, deserialize_with = "null_to_default")]
    pub genotypes: Vec<Value>,
    /// Phenotype annotations.
    #[serde(default, deserialize_with = "null_to_default")]
    pub phenotypes: Vec<Value>,
}

/// Linkage-disequilibrium statistics from the `/ld` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LDRecord {
    /// The first variant.
    #[serde(default, deserialize_with = "null_to_default")]
    pub variation1: String,
    /// The second variant.
    #[serde(default, deserialize_with = "null_to_default")]
    pub variation2: String,
    /// The D' statistic.
    pub d_prime: Option<f64>,
    /// The r-squared statistic.
    pub r2: Option<f64>,
}

/// One coordinate in a mapping result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MappedCoordinate {
    /// The sequence region name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub seq_region_name: String,
    /// The start coordinate.
    #[serde(default, deserialize_with = "null_to_default")]
    pub start: i64,
    /// The end coordinate.
    #[serde(default, deserialize_with = "null_to_default")]
    pub end: i64,
    /// The strand.
    #[serde(default, deserialize_with = "null_to_default")]
    pub strand: i64,
    /// The assembly name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub assembly: String,
    /// The coordinate system.
    #[serde(default, deserialize_with = "null_to_default")]
    pub coord_system: String,
}

/// A single original-to-mapped coordinate pair.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Mapping {
    /// The input coordinate.
    #[serde(default, deserialize_with = "null_to_default")]
    pub original: MappedCoordinate,
    /// The projected coordinate.
    #[serde(default, deserialize_with = "null_to_default")]
    pub mapped: MappedCoordinate,
}

/// The result of a `/map` call.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MappingRecord {
    /// The coordinate mappings.
    #[serde(default, deserialize_with = "null_to_default")]
    pub mappings: Vec<Mapping>,
}

/// A Variant Effect Predictor (VEP) result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct VEPRecord {
    /// The identifier the effect was predicted for.
    #[serde(default, deserialize_with = "null_to_default")]
    pub id: String,
    /// The original input line or identifier.
    #[serde(default, deserialize_with = "null_to_default")]
    pub input: String,
    /// The assembly name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub assembly_name: String,
    /// The sequence region name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub seq_region_name: String,
    /// The start coordinate.
    #[serde(default, deserialize_with = "null_to_default")]
    pub start: i64,
    /// The end coordinate.
    #[serde(default, deserialize_with = "null_to_default")]
    pub end: i64,
    /// The strand.
    #[serde(default, deserialize_with = "null_to_default")]
    pub strand: i64,
    /// The most severe predicted consequence term.
    #[serde(default, deserialize_with = "null_to_default")]
    pub most_severe_consequence: String,
    /// Per-transcript consequences.
    #[serde(default, deserialize_with = "null_to_default")]
    pub transcript_consequences: Vec<Value>,
    /// Intergenic consequences.
    #[serde(default, deserialize_with = "null_to_default")]
    pub intergenic_consequences: Vec<Value>,
    /// Known variants co-located with the input.
    #[serde(default, deserialize_with = "null_to_default")]
    pub colocated_variants: Vec<Value>,
}

/// GA4GH Beacon information from the `/ga4gh/beacon` endpoint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BeaconResponse {
    /// The beacon's identifier.
    #[serde(default, deserialize_with = "null_to_default")]
    pub id: String,
    /// The beacon's name.
    #[serde(default, deserialize_with = "null_to_default")]
    pub name: String,
    /// The GA4GH Beacon API version implemented.
    #[serde(rename = "apiVersion", default, deserialize_with = "null_to_default")]
    pub api_version: String,
    /// The organization responsible for the beacon.
    pub organization: Value,
    /// A free-text description.
    #[serde(default, deserialize_with = "null_to_default")]
    pub description: String,
    /// The beacon's version.
    #[serde(default, deserialize_with = "null_to_default")]
    pub version: String,
    /// The beacon's welcome page URL.
    #[serde(rename = "welcomeUrl", default, deserialize_with = "null_to_default")]
    pub welcome_url: String,
    /// The datasets the beacon can query.
    #[serde(default, deserialize_with = "null_to_default")]
    pub datasets: Vec<Value>,
}

/// A GA4GH Beacon allele query response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BeaconQueryResponse {
    /// The identifier of the beacon that answered the query.
    #[serde(rename = "beaconId", default, deserialize_with = "null_to_default")]
    pub beacon_id: String,
    /// Whether the queried allele was observed.
    pub exists: Option<bool>,
    /// The allele request that was evaluated.
    #[serde(rename = "alleleRequest")]
    pub allele_request: Value,
    /// Per-dataset allele responses.
    #[serde(
        rename = "datasetAlleleResponses",
        default,
        deserialize_with = "null_to_default"
    )]
    pub dataset_allele_responses: Vec<Value>,
    /// An error, when the query could not be answered.
    pub error: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_deserializes() {
        let p: PingResponse = serde_json::from_str(r#"{"ping":1}"#).unwrap();
        assert_eq!(p.ping, 1);
    }

    #[test]
    fn missing_fields_default_rather_than_failing() {
        // Ensembl omits most fields unless asked for them.
        let r: LookupRecord = serde_json::from_str(r#"{"id":"ENSG00000157764"}"#).unwrap();
        assert_eq!(r.id, "ENSG00000157764");
        assert_eq!(r.species, "");
        assert_eq!(r.start, 0);
        assert!(r.transcripts.is_empty());
    }

    #[test]
    fn snake_case_wire_names_populate_without_any_rename() {
        // These already equal their wire names, so they exercise the plain
        // (non-`#[serde(rename)]`) path, not a rename.
        let json = r#"{
            "id":"ENSG00000157764",
            "seq_region_name":"7",
            "object_type":"Gene",
            "display_name":"BRAF",
            "assembly_name":"GRCh38"
        }"#;
        let r: LookupRecord = serde_json::from_str(json).unwrap();
        assert_eq!(r.seq_region_name, "7");
        assert_eq!(r.object_type, "Gene");
        assert_eq!(r.display_name, "BRAF");
        assert_eq!(r.assembly_name, "GRCh38");
    }

    #[test]
    fn keyword_and_case_mismatched_wire_names_are_renamed() {
        // Every real #[serde(rename)] in this file, keyed on its actual wire
        // name (not the Rust field name), asserting the renamed field
        // populates. A wrong or missing rename leaves these at their
        // `#[serde(default)]` zero value with no error, so this is the only
        // thing that would catch it.
        let archive: ArchiveRecord = serde_json::from_str(r#"{"id":"a","type":"Gene"}"#).unwrap();
        assert_eq!(archive.kind, "Gene");

        let homology: HomologyRecord =
            serde_json::from_str(r#"{"type":"ortholog_one2one"}"#).unwrap();
        assert_eq!(homology.kind, "ortholog_one2one");

        let lookup: LookupRecord = serde_json::from_str(
            r#"{"id":"ENSG01","Translation":{"id":"ENSP01"},"Exon":[{"id":"ENSE01"}]}"#,
        )
        .unwrap();
        assert_eq!(lookup.translation.unwrap().id, "ENSP01");
        assert_eq!(lookup.exons[0].id, "ENSE01");

        let variation: VariationRecord = serde_json::from_str(r#"{"MAF":0.25}"#).unwrap();
        assert_eq!(variation.maf, Some(0.25));

        let beacon: BeaconResponse =
            serde_json::from_str(r#"{"apiVersion":"1.0","welcomeUrl":"https://example.org"}"#)
                .unwrap();
        assert_eq!(beacon.api_version, "1.0");
        assert_eq!(beacon.welcome_url, "https://example.org");

        let query: BeaconQueryResponse = serde_json::from_str(
            r#"{"beaconId":"b1","alleleRequest":{"a":1},"datasetAlleleResponses":[{"b":2}]}"#,
        )
        .unwrap();
        assert_eq!(query.beacon_id, "b1");
        assert_eq!(query.allele_request["a"], 1);
        assert_eq!(query.dataset_allele_responses[0]["b"], 2);
    }

    #[test]
    fn translation_recursion_via_box_round_trips() {
        // `Option<Box<LookupRecord>>` exists purely to break the recursive
        // type; this is the only test that exercises that path (the sibling
        // `Vec<LookupRecord>` recursion is covered by
        // `nested_transcripts_round_trip` below).
        let json = r#"{"id":"ENST01","Translation":{"id":"ENSP01","biotype":"protein_coding"}}"#;
        let r: LookupRecord = serde_json::from_str(json).unwrap();
        let translation = r.translation.expect("translation should be Some");
        assert_eq!(translation.id, "ENSP01");
        assert_eq!(translation.biotype, "protein_coding");
    }

    #[test]
    fn expand_extras_are_captured_rather_than_dropped() {
        // The Go port's `Extra` field is tagged `json:"-"` and can never be
        // populated. Flattening captures what ?expand=1 actually returns.
        let json = r#"{"id":"ENSG01","some_expanded_field":{"nested":true}}"#;
        let r: LookupRecord = serde_json::from_str(json).unwrap();
        assert_eq!(r.extra["some_expanded_field"]["nested"], true);
    }

    #[test]
    fn nested_transcripts_round_trip() {
        let json = r#"{"id":"ENSG01","Transcript":[{"id":"ENST01","biotype":"protein_coding"}]}"#;
        let r: LookupRecord = serde_json::from_str(json).unwrap();
        assert_eq!(r.transcripts.len(), 1);
        assert_eq!(r.transcripts[0].id, "ENST01");
        assert_eq!(r.transcripts[0].biotype, "protein_coding");
    }

    #[test]
    fn nullable_numbers_are_options() {
        let with: LDRecord =
            serde_json::from_str(r#"{"variation1":"a","variation2":"b","r2":0.8,"d_prime":null}"#)
                .unwrap();
        assert_eq!(with.r2, Some(0.8));
        assert_eq!(with.d_prime, None);
    }

    #[test]
    fn species_response_wraps_a_list() {
        let json = r#"{"species":[{"name":"homo_sapiens","display_name":"Human"}]}"#;
        let s: SpeciesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(s.species[0].name, "homo_sapiens");
        assert_eq!(s.species[0].display_name, "Human");
    }

    #[test]
    fn raw_message_fields_become_values() {
        let json = r#"{"name":"rs699","source":"dbSNP","mappings":[{"location":"1:1-1"}]}"#;
        let v: VariationRecord = serde_json::from_str(json).unwrap();
        assert_eq!(v.name, "rs699");
        assert_eq!(v.mappings[0]["location"], "1:1-1");
    }

    #[test]
    fn null_string_fields_default_rather_than_erroring() {
        // This is exactly what /info/species sends today: `strain` is null
        // for most species (including homo_sapiens), and `accession` is
        // null for some. Without `null_to_default`, decoding any one of
        // these fields as `null` fails the ENTIRE record, not just the
        // field — this reproduces the live-test failure caught against the
        // real API.
        let json = r#"{
            "name": "homo_sapiens",
            "taxon_id": null,
            "accession": null,
            "strain": null
        }"#;
        let r: SpeciesRecord = serde_json::from_str(json).unwrap();
        assert_eq!(r.name, "homo_sapiens");
        assert_eq!(r.taxon_id, "");
        assert_eq!(r.accession, "");
        assert_eq!(r.strain, "");
    }

    #[test]
    fn null_numeric_and_vec_fields_default_rather_than_erroring() {
        // Same guarantee, for a numeric field and a Vec field: a present
        // `null` must land as the type's Default, not fail the record.
        let json = r#"{"name":"homo_sapiens","release":null,"aliases":null}"#;
        let r: SpeciesRecord = serde_json::from_str(json).unwrap();
        assert_eq!(r.release, 0);
        assert!(r.aliases.is_empty());
    }

    #[test]
    fn null_tolerance_is_model_wide_not_special_cased_to_species() {
        // The same guarantee must hold on a different model, so this isn't
        // a one-off fix scoped to SpeciesRecord.
        let json = r#"{"id":"ENSG00000157764","display_name":null}"#;
        let r: LookupRecord = serde_json::from_str(json).unwrap();
        assert_eq!(r.id, "ENSG00000157764");
        assert_eq!(r.display_name, "");
    }
}
