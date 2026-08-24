//! Coordinate mapping between cDNA, CDS, protein and genomic assemblies.

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Convert from cDNA coordinates to genomic coordinates.
    pub fn get_map_cdna_to_region(
        &self,
        id: &str,
        region: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getMapCdnaToRegion",
            &[("id", id), ("region", region)],
            None,
            opts,
        )
    }

    /// Convert from CDS coordinates to genomic coordinates.
    pub fn get_map_cds_to_region(
        &self,
        id: &str,
        region: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getMapCdsToRegion",
            &[("id", id), ("region", region)],
            None,
            opts,
        )
    }

    /// Convert the coordinates of one assembly to another.
    pub fn get_map_assembly_one_to_two(
        &self,
        species: &str,
        asm_one: &str,
        region: &str,
        asm_two: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getMapAssemblyOneToTwo",
            &[
                ("species", species),
                ("asm_one", asm_one),
                ("region", region),
                ("asm_two", asm_two),
            ],
            None,
            opts,
        )
    }

    /// Convert from protein (translation) coordinates to genomic coordinates.
    pub fn get_map_translation_to_region(
        &self,
        id: &str,
        region: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getMapTranslationToRegion",
            &[("id", id), ("region", region)],
            None,
            opts,
        )
    }
}
