//! Linkage disequilibrium (LD) between variants.

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Computes and returns LD values between the given variant and all other variants in a window around it.
    pub fn get_ld_id(
        &self,
        species: &str,
        id: &str,
        population_name: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getLdId",
            &[
                ("species", species),
                ("id", id),
                ("population_name", population_name),
            ],
            None,
            opts,
        )
    }

    /// Computes and returns LD values between two given variants.
    pub fn get_ld_pairwise(
        &self,
        species: &str,
        id1: &str,
        id2: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getLdPairwise",
            &[("species", species), ("id1", id1), ("id2", id2)],
            None,
            opts,
        )
    }

    /// Computes and returns LD values between all pairs of variants in the defined region.
    pub fn get_ld_region(
        &self,
        species: &str,
        region: &str,
        population_name: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getLdRegion",
            &[
                ("species", species),
                ("region", region),
                ("population_name", population_name),
            ],
            None,
            opts,
        )
    }
}
