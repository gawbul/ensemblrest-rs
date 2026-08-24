//! Phenotype annotations for genes, regions and ontology terms.

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Returns phenotype annotations given a phenotype ontology accession.
    pub fn get_phenotype_by_accession(
        &self,
        species: &str,
        accession: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getPhenotypeByAccession",
            &[("species", species), ("accession", accession)],
            None,
            opts,
        )
    }

    /// Returns phenotype annotations for a given gene.
    pub fn get_phenotype_by_gene(
        &self,
        species: &str,
        gene: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getPhenotypeByGene",
            &[("species", species), ("gene", gene)],
            None,
            opts,
        )
    }

    /// Returns phenotype annotations that overlap a given genomic region.
    pub fn get_phenotype_by_region(
        &self,
        species: &str,
        region: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getPhenotypeByRegion",
            &[("species", species), ("region", region)],
            None,
            opts,
        )
    }

    /// Returns phenotype annotations given a phenotype ontology term.
    pub fn get_phenotype_by_term(
        &self,
        species: &str,
        term: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getPhenotypeByTerm",
            &[("species", species), ("term", term)],
            None,
            opts,
        )
    }
}
