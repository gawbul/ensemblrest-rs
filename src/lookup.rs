//! Genomic feature lookup.

use serde_json::json;

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Finds the species and database for a single identifier (e.g. gene, transcript, protein).
    pub fn get_lookup_by_id(&self, id: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getLookupById", &[("id", id)], None, opts)
    }

    /// Finds the species and database for several identifiers.
    pub fn get_lookup_by_multiple_ids(
        &self,
        ids: &[&str],
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getLookupByMultipleIds",
            &[],
            Some(&json!({ "ids": ids })),
            opts,
        )
    }

    /// Finds the species and database for a symbol in a linked external database.
    pub fn get_lookup_by_symbol(
        &self,
        species: &str,
        symbol: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getLookupBySymbol",
            &[("species", species), ("symbol", symbol)],
            None,
            opts,
        )
    }

    /// Finds the species and database for a set of symbols in a linked external database.
    pub fn get_lookup_by_multiple_symbols(
        &self,
        species: &str,
        symbols: &[&str],
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getLookupByMultipleSymbols",
            &[("species", species)],
            Some(&json!({ "symbols": symbols })),
            opts,
        )
    }
}
