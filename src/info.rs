//! Server information: species, assemblies, biotypes, comparative genomics
//! databases and data releases.

use crate::options::RequestOption;
use crate::{Client, Response, Result};

impl Client {
    /// Lists the names of analyses involved in generating Ensembl data.
    pub fn get_info_analysis(&self, species: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getInfoAnalysis", &[("species", species)], None, opts)
    }

    /// Lists the currently available assemblies for a species.
    pub fn get_info_assembly(&self, species: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getInfoAssembly", &[("species", species)], None, opts)
    }

    /// Returns information about the specified sequence region for the given species.
    pub fn get_info_assembly_region(
        &self,
        species: &str,
        region_name: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getInfoAssemblyRegion",
            &[("species", species), ("region_name", region_name)],
            None,
            opts,
        )
    }

    /// Lists functional classifications of gene models for a species.
    pub fn get_info_biotypes(&self, species: &str, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getInfoBiotypes", &[("species", species)], None, opts)
    }

    /// Lists the properties of biotypes within a group.
    pub fn get_info_biotypes_by_group(
        &self,
        group: &str,
        object_type: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getInfoBiotypesByGroup",
            &[("group", group), ("object_type", object_type)],
            None,
            opts,
        )
    }

    /// Lists the properties of biotypes with a given name.
    pub fn get_info_biotypes_by_name(
        &self,
        name: &str,
        object_type: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getInfoBiotypesByName",
            &[("name", name), ("object_type", object_type)],
            None,
            opts,
        )
    }

    /// Lists all compara analyses available.
    pub fn get_info_compara_methods(&self, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getInfoComparaMethods", &[], None, opts)
    }

    /// Lists all collections of species analysed with the specified compara method.
    pub fn get_info_compara_species_sets(
        &self,
        methods: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getInfoComparaSpeciesSets",
            &[("methods", methods)],
            None,
            opts,
        )
    }

    /// Lists all available comparative genomics databases and their data release.
    pub fn get_info_comparas(&self, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getInfoComparas", &[], None, opts)
    }

    /// Shows the data releases available on this REST server.
    pub fn get_info_data(&self, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getInfoData", &[], None, opts)
    }

    /// Returns the Ensembl Genomes version of the databases backing this service.
    pub fn get_info_eg_version(&self, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getInfoEgVersion", &[], None, opts)
    }

    /// Lists all available external sources for a species.
    pub fn get_info_external_dbs(
        &self,
        species: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getInfoExternalDbs", &[("species", species)], None, opts)
    }

    /// Gets the list of all Ensembl divisions for which information is available.
    pub fn get_info_divisions(&self, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getInfoDivisions", &[], None, opts)
    }

    /// Finds information about a given genome by name.
    pub fn get_info_genomes_by_name(
        &self,
        name: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getInfoGenomesByName", &[("name", name)], None, opts)
    }

    /// Finds information about genomes containing a specified INSDC accession.
    pub fn get_info_genomes_by_accession(
        &self,
        accession: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getInfoGenomesByAccession",
            &[("accession", accession)],
            None,
            opts,
        )
    }

    /// Finds information about a genome with a specified assembly.
    pub fn get_info_genomes_by_assembly(
        &self,
        assembly_id: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getInfoGenomesByAssembly",
            &[("assembly_id", assembly_id)],
            None,
            opts,
        )
    }

    /// Finds information about all genomes in a given division.
    pub fn get_info_genomes_by_division(
        &self,
        division: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getInfoGenomesByDivision",
            &[("division", division)],
            None,
            opts,
        )
    }

    /// Finds information about all genomes beneath a given node of the taxonomy.
    pub fn get_info_genomes_by_taxonomy(
        &self,
        taxon_name: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getInfoGenomesByTaxonomy",
            &[("taxon_name", taxon_name)],
            None,
            opts,
        )
    }

    /// Checks if the Ensembl REST service is alive.
    pub fn get_info_ping(&self, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getInfoPing", &[], None, opts)
    }

    /// Shows the current version of the Ensembl REST API.
    pub fn get_info_rest(&self, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getInfoRest", &[], None, opts)
    }

    /// Shows the current version of the Ensembl API used by the REST server.
    pub fn get_info_software(&self, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getInfoSoftware", &[], None, opts)
    }

    /// Lists all available species, aliases, and adaptor groups.
    pub fn get_info_species(&self, opts: &[RequestOption<'_>]) -> Result<Response> {
        self.call("getInfoSpecies", &[], None, opts)
    }

    /// Lists the variation sources used in Ensembl for a species.
    pub fn get_info_variation_by_species(
        &self,
        species: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getInfoVariationBySpecies",
            &[("species", species)],
            None,
            opts,
        )
    }

    /// Lists all variant consequence types.
    pub fn get_info_variation_consequence_types(
        &self,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call("getInfoVariationConsequenceTypes", &[], None, opts)
    }

    /// Lists all individuals for a population from a species.
    pub fn get_info_variation_population_individuals(
        &self,
        species: &str,
        population_name: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getInfoVariationPopulationIndividuals",
            &[("species", species), ("population_name", population_name)],
            None,
            opts,
        )
    }

    /// Lists all populations for a species.
    pub fn get_info_variation_populations(
        &self,
        species: &str,
        opts: &[RequestOption<'_>],
    ) -> Result<Response> {
        self.call(
            "getInfoVariationPopulations",
            &[("species", species)],
            None,
            opts,
        )
    }
}
