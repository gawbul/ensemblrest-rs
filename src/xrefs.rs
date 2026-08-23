//! Cross-references between Ensembl objects and external databases.

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Looks up an external symbol and returns all Ensembl objects linked to it.
    pub fn get_xrefs_by_symbol(
        &self,
        species: &str,
        symbol: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getXrefsBySymbol",
            &[("species", species), ("symbol", symbol)],
            None,
            opts,
        )
    }

    /// Performs lookups of Ensembl Identifiers and retrieves their external references in other databases.
    pub fn get_xrefs_by_id(&self, id: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getXrefsById", &[("id", id)], None, opts)
    }

    /// Performs a lookup based upon the primary accession or display label of an external reference.
    pub fn get_xrefs_by_name(
        &self,
        species: &str,
        name: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getXrefsByName",
            &[("species", species), ("name", name)],
            None,
            opts,
        )
    }
}
