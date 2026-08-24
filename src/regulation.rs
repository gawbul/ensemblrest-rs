//! Regulatory feature binding matrices.

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Returns the specified transcription factor binding matrix.
    pub fn get_regulation_binding_matrix(
        &self,
        species: &str,
        binding_matrix: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getRegulationBindingMatrix",
            &[("species", species), ("binding_matrix", binding_matrix)],
            None,
            opts,
        )
    }
}
