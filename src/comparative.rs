//! Comparative genomics: gene trees, cafe trees, alignments and homology.

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Retrieves a cafe tree of the gene tree using the gene tree stable identifier.
    pub fn get_cafe_gene_tree_by_id(
        &self,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getCafeGeneTreeById", &[("id", id)], None, opts)
    }

    /// Retrieves the cafe tree of the gene tree that contains the gene identified by a symbol.
    pub fn get_cafe_gene_tree_member_by_symbol(
        &self,
        species: &str,
        symbol: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getCafeGeneTreeMemberBySymbol",
            &[("species", species), ("symbol", symbol)],
            None,
            opts,
        )
    }

    /// Retrieves the cafe tree of the gene tree that contains the gene / transcript / translation stable identifier in the given species.
    pub fn get_cafe_gene_tree_member_by_id(
        &self,
        species: &str,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getCafeGeneTreeMemberById",
            &[("species", species), ("id", id)],
            None,
            opts,
        )
    }

    /// Retrieves a gene tree for a gene tree stable identifier.
    pub fn get_gene_tree_by_id(&self, id: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getGeneTreeById", &[("id", id)], None, opts)
    }

    /// Retrieves the gene tree that contains the gene identified by a symbol.
    pub fn get_gene_tree_member_by_symbol(
        &self,
        species: &str,
        symbol: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getGeneTreeMemberBySymbol",
            &[("species", species), ("symbol", symbol)],
            None,
            opts,
        )
    }

    /// Retrieves the gene tree that contains the gene / transcript / translation stable identifier in the given species.
    pub fn get_gene_tree_member_by_id(
        &self,
        species: &str,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getGeneTreeMemberById",
            &[("species", species), ("id", id)],
            None,
            opts,
        )
    }

    /// Retrieves genomic alignments as separate blocks based on a region and species.
    pub fn get_alignment_by_region(
        &self,
        species: &str,
        region: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getAlignmentByRegion",
            &[("species", species), ("region", region)],
            None,
            opts,
        )
    }

    /// Retrieves homology information (orthologs) by species and Ensembl gene id.
    pub fn get_homology_by_id(
        &self,
        species: &str,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getHomologyById",
            &[("species", species), ("id", id)],
            None,
            opts,
        )
    }

    /// Retrieves homology information (orthologs) by symbol.
    pub fn get_homology_by_symbol(
        &self,
        species: &str,
        symbol: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getHomologyBySymbol",
            &[("species", species), ("symbol", symbol)],
            None,
            opts,
        )
    }
}
