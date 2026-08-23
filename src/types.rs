//! Deserialization models for common Ensembl REST API responses.
//!
//! These cover the shapes the Go port models. They are a convenience, not a
//! requirement: [`crate::Response::json`] accepts any [`serde::Deserialize`]
//! type, including [`serde_json::Value`], so endpoints without a model here are
//! still fully usable.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The response from `/info/ping`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PingResponse {
    /// `1` when the service is up.
    pub ping: i64,
}

/// An archived identifier entry from the `/archive` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ArchiveRecord {
    /// The queried identifier.
    pub id: String,
    /// The latest versioned identifier.
    pub latest: String,
    /// The identifier version.
    pub version: i64,
    /// The Ensembl release this record came from.
    pub release: String,
    /// The assembly name.
    pub assembly: String,
    /// The peptide sequence, when the identifier is a translation.
    pub peptide: Option<String>,
    /// The feature type.
    #[serde(rename = "type")]
    pub kind: String,
    /// Identifiers that may replace a retired one.
    pub possible_replacement: Vec<String>,
    /// Whether this is the current version.
    pub is_current: String,
}

/// A genomic feature returned by the `/lookup` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LookupRecord {
    /// The stable identifier.
    pub id: String,
    /// The feature type, for example `"Gene"` or `"Transcript"`.
    pub object_type: String,
    /// The species name.
    pub species: String,
    /// The display label.
    pub display_name: String,
    /// A free-text description.
    pub description: String,
    /// The biotype, for example `"protein_coding"`.
    pub biotype: String,
    /// The sequence region, typically a chromosome name.
    pub seq_region_name: String,
    /// The start coordinate.
    pub start: i64,
    /// The end coordinate.
    pub end: i64,
    /// The strand, `1` or `-1`.
    pub strand: i64,
    /// The annotation source.
    pub source: String,
    /// The feature version.
    pub version: i64,
    /// The database type.
    pub db_type: String,
    /// The assembly name.
    pub assembly_name: String,
    /// The canonical transcript identifier.
    pub canonical_transcript: String,
    /// Child transcripts, populated by `?expand=1`.
    #[serde(rename = "Transcript")]
    pub transcripts: Vec<LookupRecord>,
    /// The translation, populated by `?expand=1`.
    #[serde(rename = "Translation")]
    pub translation: Option<Box<LookupRecord>>,
    /// Child exons, populated by `?expand=1`.
    #[serde(rename = "Exon")]
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
    pub id: String,
    /// The sequence itself.
    pub seq: String,
    /// A free-text description.
    pub desc: String,
    /// The original query string, for region-based lookups.
    pub query: String,
    /// The sequence version.
    pub version: i64,
    /// The molecule type, for example `"dna"` or `"protein"`.
    pub molecule: String,
}

/// Species metadata from `/info/species`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SpeciesRecord {
    /// The production name.
    pub name: String,
    /// The human-readable name.
    pub display_name: String,
    /// The NCBI taxonomy identifier.
    pub taxon_id: String,
    /// The species name.
    pub species: String,
    /// The common name.
    pub common_name: String,
    /// The Ensembl division.
    pub division: String,
    /// The Ensembl release.
    pub release: i64,
    /// The assembly name.
    pub assembly: String,
    /// The assembly accession.
    pub accession: String,
    /// Alternative names.
    pub aliases: Vec<String>,
    /// Group memberships.
    pub groups: Vec<String>,
    /// The strain, where applicable.
    pub strain: String,
    /// The strain type.
    pub strain_type: String,
    /// Whether this is the reference genome for its group.
    pub is_reference: i64,
    /// Whether pan-taxonomic comparative data is available.
    pub has_pan_compara: i64,
    /// Whether variation data is available.
    pub has_variations: i64,
    /// Whether peptide comparative data is available.
    pub has_peptide_compara: i64,
    /// Whether whole-genome alignments are available.
    pub has_genome_alignments: i64,
    /// Whether synteny data is available.
    pub has_synteny: i64,
}

/// The wrapper `/info/species` returns.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SpeciesResponse {
    /// The species records.
    pub species: Vec<SpeciesRecord>,
}

/// Assembly information from `/info/assembly`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AssemblyInfo {
    /// The date the assembly was produced.
    pub assembly_date: String,
    /// The assembly level, for example `"chromosome"`.
    pub assembly_level: String,
    /// The assembly's INSDC accession.
    pub assembly_accession: String,
    /// The assembly name.
    pub assembly_name: String,
    /// The default coordinate system version.
    pub default_coord_system_version: String,
    /// The date the gene build was produced.
    pub genebuild_date: String,
    /// The gene build method.
    pub genebuild_method: String,
    /// The date the gene build started.
    pub genebuild_start_date: String,
    /// The gene build version.
    pub genebuild_version: String,
    /// The names of the top-level sequence regions.
    pub top_level_region: Vec<String>,
    /// The karyotype, as an ordered list of chromosome names.
    pub karyotype: Vec<String>,
    /// The coordinate system versions available for this assembly.
    pub coord_system_versions: Vec<Value>,
}

/// Region metadata from `/info/assembly/:species/:region_name`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AssemblyRegionInfo {
    /// The region's length, in base pairs.
    pub length: i64,
    /// The coordinate system this region belongs to.
    pub coord_system: String,
    /// `1` when the region is a chromosome.
    pub is_chromosome: i64,
    /// `1` when the region is circular.
    pub is_circular: i64,
    /// The assembly name.
    pub assembly: String,
    /// The sequence region name.
    pub seq_region_name: String,
}

/// One homologous relationship between two genes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HomologyRecord {
    /// The homology type, for example `"ortholog_one2one"`.
    #[serde(rename = "type")]
    pub kind: String,
    /// The comparative analysis method used to call the homology.
    pub method_link_type: String,
    /// The taxonomic level the homology was called at.
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
    pub id: String,
    /// The homologous genes found.
    pub homologies: Vec<HomologyRecord>,
    /// The last common ancestor taxon, when known.
    pub last_common_ancestor: String,
}

/// The response from the `/homology` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HomologyResponse {
    /// One entry per queried gene.
    pub data: Vec<HomologyData>,
}

/// An external reference from the `/xrefs` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct XrefRecord {
    /// The external identifier.
    pub id: String,
    /// The external database name.
    pub dbname: String,
    /// The display identifier.
    pub display_id: String,
    /// The primary identifier.
    pub primary_id: String,
    /// A free-text description.
    pub description: String,
    /// Known synonyms.
    pub synonyms: Vec<String>,
    /// The type of cross-reference information.
    pub info_type: String,
    /// Free-text information about the cross-reference.
    pub info_text: String,
}

/// A genetic variant from the `/variation` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct VariationRecord {
    /// The variant name, for example `"rs699"`.
    pub name: String,
    /// The source database.
    pub source: String,
    /// The variant class.
    pub var_class: String,
    /// The most severe consequence term.
    pub most_severe_consequence: String,
    /// The minor allele frequency.
    #[serde(rename = "MAF")]
    pub maf: Option<f64>,
    /// The minor allele.
    pub minor_allele: String,
    /// Supporting evidence terms.
    pub evidence: Vec<String>,
    /// Known synonyms.
    pub synonyms: Vec<String>,
    /// Clinical significance terms.
    pub clinical_significance: Vec<String>,
    /// Genomic mappings.
    pub mappings: Vec<Value>,
    /// Genotype records.
    pub genotypes: Vec<Value>,
    /// Phenotype annotations.
    pub phenotypes: Vec<Value>,
}

/// Linkage-disequilibrium statistics from the `/ld` endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LDRecord {
    /// The first variant.
    pub variation1: String,
    /// The second variant.
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
    pub seq_region_name: String,
    /// The start coordinate.
    pub start: i64,
    /// The end coordinate.
    pub end: i64,
    /// The strand.
    pub strand: i64,
    /// The assembly name.
    pub assembly: String,
    /// The coordinate system.
    pub coord_system: String,
}

/// A single original-to-mapped coordinate pair.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Mapping {
    /// The input coordinate.
    pub original: MappedCoordinate,
    /// The projected coordinate.
    pub mapped: MappedCoordinate,
}

/// The result of a `/map` call.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MappingRecord {
    /// The coordinate mappings.
    pub mappings: Vec<Mapping>,
}

/// A Variant Effect Predictor (VEP) result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct VEPRecord {
    /// The identifier the effect was predicted for.
    pub id: String,
    /// The original input line or identifier.
    pub input: String,
    /// The assembly name.
    pub assembly_name: String,
    /// The sequence region name.
    pub seq_region_name: String,
    /// The start coordinate.
    pub start: i64,
    /// The end coordinate.
    pub end: i64,
    /// The strand.
    pub strand: i64,
    /// The most severe predicted consequence term.
    pub most_severe_consequence: String,
    /// Per-transcript consequences.
    pub transcript_consequences: Vec<Value>,
    /// Intergenic consequences.
    pub intergenic_consequences: Vec<Value>,
    /// Known variants co-located with the input.
    pub colocated_variants: Vec<Value>,
}

/// GA4GH Beacon information from the `/ga4gh/beacon` endpoint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BeaconResponse {
    /// The beacon's identifier.
    pub id: String,
    /// The beacon's name.
    pub name: String,
    /// The GA4GH Beacon API version implemented.
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    /// The organization responsible for the beacon.
    pub organization: Value,
    /// A free-text description.
    pub description: String,
    /// The beacon's version.
    pub version: String,
    /// The beacon's welcome page URL.
    #[serde(rename = "welcomeUrl")]
    pub welcome_url: String,
    /// The datasets the beacon can query.
    pub datasets: Vec<Value>,
}

/// A GA4GH Beacon allele query response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BeaconQueryResponse {
    /// The identifier of the beacon that answered the query.
    #[serde(rename = "beaconId")]
    pub beacon_id: String,
    /// Whether the queried allele was observed.
    pub exists: Option<bool>,
    /// The allele request that was evaluated.
    #[serde(rename = "alleleRequest")]
    pub allele_request: Value,
    /// Per-dataset allele responses.
    #[serde(rename = "datasetAlleleResponses")]
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
    fn wire_names_that_differ_from_field_names_are_renamed() {
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
}
