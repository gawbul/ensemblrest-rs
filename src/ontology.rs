//! Ontology terms and taxonomy classification.

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Reconstructs the entire ancestry of an ontology term from is_a and part_of relationships.
    pub fn get_ancestors_by_id(&self, id: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getAncestorsById", &[("id", id)], None, opts)
    }

    /// Reconstructs the entire ancestry chart of a term.
    pub fn get_ancestors_chart_by_id(
        &self,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getAncestorsChartById", &[("id", id)], None, opts)
    }

    /// Finds all terms descended from a given term.
    pub fn get_descendants_by_id(&self, id: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getDescendantsById", &[("id", id)], None, opts)
    }

    /// Searches for an ontological term by its namespaced identifier.
    pub fn get_ontology_by_id(&self, id: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getOntologyById", &[("id", id)], None, opts)
    }

    /// Searches for a list of ontological terms by their name.
    pub fn get_ontology_by_name(&self, name: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getOntologyByName", &[("name", name)], None, opts)
    }

    /// Returns the taxonomic classification of a taxon node.
    pub fn get_taxonomy_classification_by_id(
        &self,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getTaxonomyClassificationById", &[("id", id)], None, opts)
    }

    /// Searches for a taxonomic term by its identifier or name.
    pub fn get_taxonomy_by_id(&self, id: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getTaxonomyById", &[("id", id)], None, opts)
    }

    /// Searches for a taxonomic id by a non-scientific name.
    pub fn get_taxonomy_by_name(&self, name: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getTaxonomyByName", &[("name", name)], None, opts)
    }
}
