//! Transcript haplotypes from phased genotype data.

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Computes observed transcript haplotype sequences based on phased genotype data.
    pub fn get_transcript_haplotypes(
        &self,
        species: &str,
        id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getTranscriptHaplotypes",
            &[("species", species), ("id", id)],
            None,
            opts,
        )
    }
}
