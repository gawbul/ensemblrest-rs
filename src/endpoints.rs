//! The catalog of Ensembl REST API endpoints and dynamic dispatch.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde_json::Value;

use crate::error::{Error, Result};
use crate::options::{RequestOption, resolve};
use crate::response::Response;
use crate::{Client, DEFAULT_CONTENT_TYPE};

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
    /// Prefer the typed endpoint methods, or [`Client::call`] for an endpoint
    /// that is in the table; this exists for this crate's own tests and for
    /// callers working with paths not yet in the endpoint table. It is
    /// `#[doc(hidden)]` and outside the crate's semver promise, and the design
    /// spec does not describe it.
    ///
    /// Note the one behavioural difference from [`Client::call`]: with no
    /// endpoint entry there is no `spec.content_type` to start from, so this
    /// defaults to [`crate::DEFAULT_CONTENT_TYPE`]. Any endpoint whose table
    /// entry names a different content type must be given one explicitly via
    /// [`crate::content_type`] here.
    #[doc(hidden)]
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

/// Metadata describing one Ensembl REST API endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndpointSpec {
    /// The camelCase name, shared with `goensemblrest` and `pyEnsemblRest`.
    pub name: &'static str,
    /// A one-line description, taken from the Ensembl documentation.
    pub doc: &'static str,
    /// The path template, with `{{param}}` placeholders.
    pub url: &'static str,
    /// The HTTP method.
    pub method: Method,
    /// The default `Content-Type` and `Accept` value.
    pub content_type: &'static str,
    /// The JSON body keys this endpoint accepts, for POST endpoints.
    pub post_parameters: &'static [&'static str],
}

/// The full catalog of Ensembl REST API endpoints.
///
/// Ported entry-for-entry from `goensemblrest`'s `EndpointsTable`, preserving
/// its order and grouping.
pub static ENDPOINTS: &[EndpointSpec] = &[
    // ---- Archive ----
    EndpointSpec {
        name: "getArchiveById",
        doc: "Uses the given identifier to return its latest version",
        url: "/archive/id/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getArchiveByMultipleIds",
        doc: "Retrieve the latest version for a set of identifiers",
        url: "/archive/id",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["id"],
    },
    // ---- Comparative Genomics ----
    EndpointSpec {
        name: "getCafeGeneTreeById",
        doc: "Retrieves a cafe tree of the gene tree using the gene tree stable identifier",
        url: "/cafe/genetree/id/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getCafeGeneTreeMemberBySymbol",
        doc: "Retrieves the cafe tree of the gene tree that contains the gene identified by a symbol",
        url: "/cafe/genetree/member/symbol/{{species}}/{{symbol}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getCafeGeneTreeMemberById",
        doc: "Retrieves the cafe tree of the gene tree that contains the gene / transcript / translation stable identifier in the given species",
        url: "/cafe/genetree/member/id/{{species}}/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getGeneTreeById",
        doc: "Retrieves a gene tree for a gene tree stable identifier",
        url: "/genetree/id/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getGeneTreeMemberBySymbol",
        doc: "Retrieves the gene tree that contains the gene identified by a symbol",
        url: "/genetree/member/symbol/{{species}}/{{symbol}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getGeneTreeMemberById",
        doc: "Retrieves the gene tree that contains the gene / transcript / translation stable identifier in the given species",
        url: "/genetree/member/id/{{species}}/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getAlignmentByRegion",
        doc: "Retrieves genomic alignments as separate blocks based on a region and species",
        url: "/alignment/region/{{species}}/{{region}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getHomologyById",
        doc: "Retrieves homology information (orthologs) by species and Ensembl gene id",
        url: "/homology/id/{{species}}/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getHomologyBySymbol",
        doc: "Retrieves homology information (orthologs) by symbol",
        url: "/homology/symbol/{{species}}/{{symbol}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    // ---- Cross References ----
    EndpointSpec {
        name: "getXrefsBySymbol",
        doc: "Looks up an external symbol and returns all Ensembl objects linked to it",
        url: "/xrefs/symbol/{{species}}/{{symbol}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getXrefsById",
        doc: "Perform lookups of Ensembl Identifiers and retrieve their external references in other databases",
        url: "/xrefs/id/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getXrefsByName",
        doc: "Performs a lookup based upon the primary accession or display label of an external reference",
        url: "/xrefs/name/{{species}}/{{name}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    // ---- Information ----
    EndpointSpec {
        name: "getInfoAnalysis",
        doc: "List the names of analyses involved in generating Ensembl data",
        url: "/info/analysis/{{species}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoAssembly",
        doc: "List the currently available assemblies for a species, along with toplevel sequences, chromosomes and cytogenetic bands",
        url: "/info/assembly/{{species}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoAssemblyRegion",
        doc: "Returns information about the specified toplevel sequence region for the given species",
        url: "/info/assembly/{{species}}/{{region_name}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoBiotypes",
        doc: "List the functional classifications of gene models that Ensembl associates with a particular species",
        url: "/info/biotypes/{{species}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoBiotypesByGroup",
        doc: "List the properties of biotypes within a group",
        url: "/info/biotypes/groups/{{group}}/{{object_type}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoBiotypesByName",
        doc: "List the properties of biotypes with a given name",
        url: "/info/biotypes/name/{{name}}/{{object_type}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoComparaMethods",
        doc: "List all compara analyses available",
        url: "/info/compara/methods",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoComparaSpeciesSets",
        doc: "List all collections of species analysed with the specified compara method",
        url: "/info/compara/species_sets/{{methods}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoComparas",
        doc: "Lists all available comparative genomics databases and their data release",
        url: "/info/comparas",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoData",
        doc: "Shows the data releases available on this REST server",
        url: "/info/data",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoEgVersion",
        doc: "Returns the Ensembl Genomes version of the databases backing this service",
        url: "/info/eg_version",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoExternalDbs",
        doc: "Lists all available external sources for a species",
        url: "/info/external_dbs/{{species}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoDivisions",
        doc: "Get list of all Ensembl divisions for which information is available",
        url: "/info/divisions",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoGenomesByName",
        doc: "Find information about a given genome",
        url: "/info/genomes/{{name}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoGenomesByAccession",
        doc: "Find information about genomes containing a specified INSDC accession",
        url: "/info/genomes/accession/{{accession}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoGenomesByAssembly",
        doc: "Find information about a genome with a specified assembly",
        url: "/info/genomes/assembly/{{assembly_id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoGenomesByDivision",
        doc: "Find information about all genomes in a given division",
        url: "/info/genomes/division/{{division}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoGenomesByTaxonomy",
        doc: "Find information about all genomes beneath a given node of the taxonomy",
        url: "/info/genomes/taxonomy/{{taxon_name}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoPing",
        doc: "Checks if the service is alive",
        url: "/info/ping",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoRest",
        doc: "Shows the current version of the Ensembl REST API",
        url: "/info/rest",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoSoftware",
        doc: "Shows the current version of the Ensembl API used by the REST server",
        url: "/info/software",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoSpecies",
        doc: "Lists all available species, their aliases, available adaptor groups and data release",
        url: "/info/species",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoVariationBySpecies",
        doc: "List the variation sources used in Ensembl for a species",
        url: "/info/variation/{{species}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoVariationConsequenceTypes",
        doc: "Lists all variant consequence types",
        url: "/info/variation/consequence_types",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoVariationPopulationIndividuals",
        doc: "List all individuals for a population from a species",
        url: "/info/variation/populations/{{species}}/{{population_name}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getInfoVariationPopulations",
        doc: "List all populations for a species",
        url: "/info/variation/populations/{{species}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    // ---- Linkage Disequilibrium ----
    EndpointSpec {
        name: "getLdId",
        doc: "Computes and returns LD values between the given variant and all other variants in a window centered around the given variant",
        url: "/ld/{{species}}/{{id}}/{{population_name}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getLdPairwise",
        doc: "Computes and returns LD values between the given variants",
        url: "/ld/{{species}}/pairwise/{{id1}}/{{id2}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getLdRegion",
        doc: "Computes and returns LD values between all pairs of variants in the defined region",
        url: "/ld/{{species}}/region/{{region}}/{{population_name}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    // ---- Lookup ----
    EndpointSpec {
        name: "getLookupById",
        doc: "Find the species and database for a single identifier e.g. gene, transcript, protein",
        url: "/lookup/id/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getLookupByMultipleIds",
        doc: "Find the species and database for several identifiers",
        url: "/lookup/id",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["ids"],
    },
    EndpointSpec {
        name: "getLookupBySymbol",
        doc: "Find the species and database for a symbol in a linked external database",
        url: "/lookup/symbol/{{species}}/{{symbol}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getLookupByMultipleSymbols",
        doc: "Find the species and database for a set of symbols in a linked external database",
        url: "/lookup/symbol/{{species}}",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["symbols"],
    },
    // ---- Mapping ----
    EndpointSpec {
        name: "getMapCdnaToRegion",
        doc: "Convert from cDNA coordinates to genomic coordinates",
        url: "/map/cdna/{{id}}/{{region}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getMapCdsToRegion",
        doc: "Convert from CDS coordinates to genomic coordinates",
        url: "/map/cds/{{id}}/{{region}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getMapAssemblyOneToTwo",
        doc: "Convert the coordinates of one assembly to another",
        url: "/map/{{species}}/{{asm_one}}/{{region}}/{{asm_two}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getMapTranslationToRegion",
        doc: "Convert from protein (translation) coordinates to genomic coordinates",
        url: "/map/translation/{{id}}/{{region}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    // ---- Ontologies and Taxonomy ----
    EndpointSpec {
        name: "getAncestorsById",
        doc: "Reconstruct the entire ancestry of a term from is_a and part_of relationships",
        url: "/ontology/ancestors/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getAncestorsChartById",
        doc: "Reconstruct the entire ancestry of a term from is_a and part_of relationships",
        url: "/ontology/ancestors/chart/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getDescendantsById",
        doc: "Find all the terms descended from a given term",
        url: "/ontology/descendants/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getOntologyById",
        doc: "Search for an ontological term by its namespaced identifier",
        url: "/ontology/id/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getOntologyByName",
        doc: "Search for a list of ontological terms by their name",
        url: "/ontology/name/{{name}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getTaxonomyClassificationById",
        doc: "Return the taxonomic classification of a taxon node",
        url: "/taxonomy/classification/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getTaxonomyById",
        doc: "Search for a taxonomic term by its identifier or name",
        url: "/taxonomy/id/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getTaxonomyByName",
        doc: "Search for a taxonomic id by a non-scientific name",
        url: "/taxonomy/name/{{name}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    // ---- Overlap ----
    EndpointSpec {
        name: "getOverlapById",
        doc: "Retrieves features that overlap a region defined by the given identifier",
        url: "/overlap/id/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getOverlapByRegion",
        doc: "Retrieves features that overlap a given region",
        url: "/overlap/region/{{species}}/{{region}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getOverlapByTranslation",
        doc: "Retrieve features related to a specific Translation as described by its stable ID",
        url: "/overlap/translation/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    // ---- Phenotype annotations ----
    EndpointSpec {
        name: "getPhenotypeByAccession",
        doc: "Return phenotype annotations for genomic features given a phenotype ontology accession",
        url: "/phenotype/accession/{{species}}/{{accession}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getPhenotypeByGene",
        doc: "Return phenotype annotations for a given gene",
        url: "/phenotype/gene/{{species}}/{{gene}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getPhenotypeByRegion",
        doc: "Return phenotype annotations that overlap a given genomic region",
        url: "/phenotype/region/{{species}}/{{region}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getPhenotypeByTerm",
        doc: "Return phenotype annotations for genomic features given a phenotype ontology term",
        url: "/phenotype/term/{{species}}/{{term}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    // ---- Regulation ----
    EndpointSpec {
        name: "getRegulationBindingMatrix",
        doc: "Return the specified binding matrix",
        url: "/species/{{species}}/binding_matrix/{{binding_matrix}}/",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    // ---- Sequences ----
    EndpointSpec {
        name: "getSequenceById",
        doc: "Request multiple types of sequence by stable identifier",
        url: "/sequence/id/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getSequenceByMultipleIds",
        doc: "Request multiple types of sequence by a stable identifier list",
        url: "/sequence/id",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["ids"],
    },
    EndpointSpec {
        name: "getSequenceByRegion",
        doc: "Returns the genomic sequence of the specified region of the given species",
        url: "/sequence/region/{{species}}/{{region}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getSequenceByMultipleRegions",
        doc: "Request multiple types of sequence by a list of regions",
        url: "/sequence/region/{{species}}",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["regions"],
    },
    // ---- Transcript Haplotypes ----
    EndpointSpec {
        name: "getTranscriptHaplotypes",
        doc: "Computes observed transcript haplotype sequences based on phased genotype data",
        url: "/transcript_haplotypes/{{species}}/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    // ---- VEP ----
    EndpointSpec {
        name: "getVariantConsequencesByHGVSNotation",
        doc: "Fetch variant consequences based on a HGVS notation",
        url: "/vep/{{species}}/hgvs/{{hgvs_notation}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getVariantConsequencesByMultipleHGVSNotations",
        doc: "Fetch variant consequences for multiple HGVS notations",
        url: "/vep/{{species}}/hgvs/",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["hgvs_notations"],
    },
    EndpointSpec {
        name: "getVariantConsequencesById",
        doc: "Fetch variant consequences based on a variant identifier",
        url: "/vep/{{species}}/id/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getVariantConsequencesByMultipleIds",
        doc: "Fetch variant consequences for multiple ids",
        url: "/vep/{{species}}/id",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["ids"],
    },
    EndpointSpec {
        name: "getVariantConsequencesByRegion",
        doc: "Fetch variant consequences for a region and allele",
        url: "/vep/{{species}}/region/{{region}}/{{allele}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getVariantConsequencesByMultipleRegions",
        doc: "Fetch variant consequences for multiple regions",
        url: "/vep/{{species}}/region",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["variants"],
    },
    // ---- Variation ----
    EndpointSpec {
        name: "getVariationRecoderById",
        doc: "Translate a variant identifier, HGVS notation or genomic SPDI notation to all possible variant IDs, HGVS and genomic SPDI",
        url: "/variant_recoder/{{species}}/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getVariationRecoderByMultipleIds",
        doc: "Translate a list of variant identifiers, HGVS notations or genomic SPDI notations to all possible variant IDs, HGVS and genomic SPDI",
        url: "/variant_recoder/{{species}}",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["ids"],
    },
    EndpointSpec {
        name: "getVariationById",
        doc: "Uses a variant identifier (e.g. rsID) to return the variation features including optional genotype, phenotype and population data",
        url: "/variation/{{species}}/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getVariationByPMCID",
        doc: "Uses a variant identifier to return variation features with PMCID data",
        url: "/variation/{{species}}/pmcid/{{pmcid}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getVariationByPMID",
        doc: "Uses a variant identifier to return variation features with PMID data",
        url: "/variation/{{species}}/pmid/{{pmid}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getVariationByMultipleIds",
        doc: "Uses a list of variant identifiers to return variation features",
        url: "/variation/{{species}}",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["ids"],
    },
    // ---- GA4GH ----
    EndpointSpec {
        name: "getGA4GHBeacon",
        doc: "Return Beacon information",
        url: "/ga4gh/beacon",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getGA4GHBeaconQuery",
        doc: "Return the Beacon response for allele information",
        url: "/ga4gh/beacon/query",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "postGA4GHBeaconQuery",
        doc: "Return the Beacon response for allele information",
        url: "/ga4gh/beacon/query",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[
            "alternateBases",
            "assemblyId",
            "end",
            "referenceBases",
            "referenceName",
            "start",
            "variantType",
        ],
    },
    EndpointSpec {
        name: "getGA4GHFeaturesById",
        doc: "Return the GA4GH record for a specific sequence feature given its identifier",
        url: "/ga4gh/features/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "searchGA4GHFeatures",
        doc: "Return a list of sequence annotation features in GA4GH format",
        url: "/ga4gh/features/search",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["end", "referenceName", "start", "featureSetId", "parentId"],
    },
    EndpointSpec {
        name: "searchGA4GHCallset",
        doc: "Return a list of sets of genotype calls for specific samples in GA4GH format",
        url: "/ga4gh/callsets/search",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["variantSetId", "name", "pageToken", "pageSize"],
    },
    EndpointSpec {
        name: "getGA4GHCallsetById",
        doc: "Return the GA4GH record for a specific CallSet given its identifier",
        url: "/ga4gh/callsets/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "searchGA4GHDatasets",
        doc: "Return a list of datasets in GA4GH format",
        url: "/ga4gh/datasets/search",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["pageToken", "pageSize"],
    },
    EndpointSpec {
        name: "getGA4GHDatasetsById",
        doc: "Return the GA4GH record for a specific dataset given its identifier",
        url: "/ga4gh/datasets/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "searchGA4GHFeaturesets",
        doc: "Return a list of feature sets in GA4GH format",
        url: "/ga4gh/featuresets/search",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["datasetId", "pageToken", "pageSize"],
    },
    EndpointSpec {
        name: "getGA4GHFeaturesetsById",
        doc: "Return the GA4GH record for a specific featureSet given its identifier",
        url: "/ga4gh/featuresets/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "getGA4GHVariantsById",
        doc: "Return the GA4GH record for a specific variant given its identifier",
        url: "/ga4gh/variants/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "searchGA4GHVariantAnnotations",
        doc: "Return variant annotation information in GA4GH format for a region on a reference sequence",
        url: "/ga4gh/variantannotations/search",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[
            "variantAnnotationSetId",
            "effects",
            "end",
            "pageSize",
            "pageToken",
            "referenceId",
            "referenceName",
            "start",
        ],
    },
    EndpointSpec {
        name: "searchGA4GHVariants",
        doc: "Return variant call information in GA4GH format for a region on a reference sequence",
        url: "/ga4gh/variants/search",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[
            "variantSetId",
            "callSetIds",
            "referenceName",
            "start",
            "end",
            "pageToken",
            "pageSize",
        ],
    },
    EndpointSpec {
        name: "searchGA4GHVariantsets",
        doc: "Return a list of variant sets in GA4GH format",
        url: "/ga4gh/variantsets/search",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["datasetId", "pageToken", "pageSize"],
    },
    EndpointSpec {
        name: "getGA4GHVariantsetsById",
        doc: "Return the GA4GH record for a specific VariantSet given its identifier",
        url: "/ga4gh/variantsets/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "searchGA4GHReferences",
        doc: "Return a list of reference sequences in GA4GH format",
        url: "/ga4gh/references/search",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[
            "referenceSetId",
            "md5checksum",
            "accession",
            "pageToken",
            "pageSize",
        ],
    },
    EndpointSpec {
        name: "getGA4GHReferencesById",
        doc: "Return data for a specific reference in GA4GH format by id",
        url: "/ga4gh/references/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "searchGA4GHReferencesets",
        doc: "Return a list of reference sets in GA4GH format",
        url: "/ga4gh/referencesets/search",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["accession", "pageToken", "pageSize"],
    },
    EndpointSpec {
        name: "getGA4GHReferencesetsById",
        doc: "Return data for a specific reference set in GA4GH format",
        url: "/ga4gh/referencesets/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
    EndpointSpec {
        name: "searchGA4GHVariantAnnotationsets",
        doc: "Return a list of annotation sets in GA4GH format",
        url: "/ga4gh/variantannotationsets/search",
        method: Method::Post,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &["variantSetId", "pageToken", "pageSize"],
    },
    EndpointSpec {
        name: "getGA4GHVariantAnnotationsetsById",
        doc: "Return meta data for a specific annotation set in GA4GH format",
        url: "/ga4gh/variantannotationsets/{{id}}",
        method: Method::Get,
        content_type: DEFAULT_CONTENT_TYPE,
        post_parameters: &[],
    },
];

/// Returns the endpoint catalog, indexed by name.
///
/// Unlike the Go port, which clones its map defensively, this returns a shared
/// reference: the data is immutable `'static`, so there is nothing to defend.
pub fn endpoints() -> &'static HashMap<&'static str, &'static EndpointSpec> {
    static INDEX: OnceLock<HashMap<&'static str, &'static EndpointSpec>> = OnceLock::new();
    INDEX.get_or_init(|| ENDPOINTS.iter().map(|e| (e.name, e)).collect())
}

/// Looks up a single endpoint by name.
pub fn endpoint(name: &str) -> Option<&'static EndpointSpec> {
    endpoints().get(name).copied()
}

impl Client {
    /// Calls any endpoint by its camelCase name.
    ///
    /// This is the equivalent of the Go port's `Call` and of `pyEnsemblRest`'s
    /// dynamic attribute dispatch. Prefer the typed methods where one exists.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnknownEndpoint`] if `name` is not in [`ENDPOINTS`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ensemblrest::Client;
    /// # use ensemblrest::serde_json;
    /// # fn main() -> ensemblrest::Result<()> {
    /// let client = Client::new()?;
    /// let v: serde_json::Value = client
    ///     .call("getLookupById", &[("id", "ENSG00000157764")], None, &[])?
    ///     .json()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn call(
        &self,
        name: &str,
        path_params: &[(&str, &str)],
        body: Option<&Value>,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        let spec = endpoint(name).ok_or_else(|| Error::UnknownEndpoint(name.to_string()))?;
        let cfg = resolve(spec.content_type, opts);
        self.execute(spec.method, spec.url, path_params, body, &cfg)
    }
}
