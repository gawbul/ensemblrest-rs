//! Offline tests for every typed endpoint method.
//!
//! Each test asserts the HTTP method, resolved path and request body that a
//! typed method produces, against the std-only mock server.

mod common;

use common::mock::MockServer;
use ensemblrest::Client;
use ensemblrest::serde_json::json;

/// A client pointed at a mock server returning `{}` once.
fn client(server: &MockServer) -> Client {
    Client::builder()
        .base_url(server.base_url())
        .build()
        .unwrap()
}

// ---- archive ----

#[test]
fn get_archive_by_id() {
    let server = MockServer::with_json(200, r#"{"id":"ENSG00000157764"}"#);
    client(&server)
        .get_archive_by_id("ENSG00000157764", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/archive/id/ENSG00000157764");
}

#[test]
fn get_archive_by_multiple_ids_uses_the_id_key_not_ids() {
    // Copied from the Go source: archive is the odd one out.
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_archive_by_multiple_ids(&["ENSG00000157764"], &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/archive/id");
    assert_eq!(req.json()["id"][0], "ENSG00000157764");
    assert!(
        req.json().get("ids").is_none(),
        "archive must send 'id', not 'ids'"
    );
}

// ---- lookup ----

#[test]
fn get_lookup_by_id() {
    let server = MockServer::with_json(200, r#"{"id":"ENSG00000157764"}"#);
    client(&server)
        .get_lookup_by_id("ENSG00000157764", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/lookup/id/ENSG00000157764");
}

#[test]
fn get_lookup_by_symbol() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_lookup_by_symbol("homo_sapiens", "BRAF", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/lookup/symbol/homo_sapiens/BRAF");
}

#[test]
fn get_lookup_by_multiple_ids() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_lookup_by_multiple_ids(&["ENSG00000157764", "ENSG00000248378"], &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/lookup/id");
    assert_eq!(req.json()["ids"][0], "ENSG00000157764");
    assert_eq!(req.json()["ids"][1], "ENSG00000248378");
}

#[test]
fn get_lookup_by_multiple_symbols() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_lookup_by_multiple_symbols("homo_sapiens", &["BRAF", "BRCA2"], &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/lookup/symbol/homo_sapiens");
    assert_eq!(req.json()["symbols"][0], "BRAF");
    assert_eq!(req.json()["symbols"][1], "BRCA2");
}

// ---- sequence ----

#[test]
fn get_sequence_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_sequence_by_id("ENSG00000157764", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/sequence/id/ENSG00000157764");
}

#[test]
fn get_sequence_by_multiple_ids() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_sequence_by_multiple_ids(&["ENSG00000157764", "ENSG00000248378"], &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/sequence/id");
    assert_eq!(req.json()["ids"][0], "ENSG00000157764");
    assert_eq!(req.json()["ids"][1], "ENSG00000248378");
}

#[test]
fn get_sequence_by_region_preserves_colons() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_sequence_by_region("homo_sapiens", "X:1000000..1000100:1", &[])
        .unwrap();

    assert_eq!(
        server.only_request().path(),
        "/sequence/region/homo_sapiens/X:1000000..1000100:1"
    );
}

#[test]
fn get_sequence_by_multiple_regions() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_sequence_by_multiple_regions("homo_sapiens", &["X:1000000..1000100:1"], &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/sequence/region/homo_sapiens");
    assert_eq!(req.json()["regions"][0], "X:1000000..1000100:1");
}

// ---- xrefs ----

#[test]
fn get_xrefs_by_symbol() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_xrefs_by_symbol("homo_sapiens", "BRAF", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/xrefs/symbol/homo_sapiens/BRAF");
}

#[test]
fn get_xrefs_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_xrefs_by_id("ENSG00000157764", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/xrefs/id/ENSG00000157764");
}

#[test]
fn get_xrefs_by_name() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_xrefs_by_name("homo_sapiens", "BRCA2", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/xrefs/name/homo_sapiens/BRCA2");
}

// ---- mapping ----

#[test]
fn get_map_cdna_to_region() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_map_cdna_to_region("ENST00000288602", "100..300", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/map/cdna/ENST00000288602/100..300");
}

#[test]
fn get_map_cds_to_region() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_map_cds_to_region("ENST00000288602", "1..300", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/map/cds/ENST00000288602/1..300");
}

#[test]
fn get_map_assembly_one_to_two() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_map_assembly_one_to_two(
            "homo_sapiens",
            "GRCh37",
            "X:1000000..1000100:1",
            "GRCh38",
            &[],
        )
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(
        req.path(),
        "/map/homo_sapiens/GRCh37/X:1000000..1000100:1/GRCh38"
    );
}

#[test]
fn get_map_translation_to_region() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_map_translation_to_region("ENSP00000288602", "100..300", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/map/translation/ENSP00000288602/100..300");
}

// ---- overlap ----

#[test]
fn get_overlap_by_id() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_overlap_by_id("ENSG00000157764", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/overlap/id/ENSG00000157764");
}

#[test]
fn get_overlap_by_region() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_overlap_by_region("homo_sapiens", "X:1000000..1000100:1", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(
        req.path(),
        "/overlap/region/homo_sapiens/X:1000000..1000100:1"
    );
}

#[test]
fn get_overlap_by_translation() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_overlap_by_translation("ENSP00000288602", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/overlap/translation/ENSP00000288602");
}

// ---- comparative ----

#[test]
fn get_cafe_gene_tree_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_cafe_gene_tree_by_id("ENSGT00390000003602", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/cafe/genetree/id/ENSGT00390000003602");
}

#[test]
fn get_cafe_gene_tree_member_by_symbol() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_cafe_gene_tree_member_by_symbol("homo_sapiens", "BRCA2", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(
        req.path(),
        "/cafe/genetree/member/symbol/homo_sapiens/BRCA2"
    );
}

#[test]
fn get_cafe_gene_tree_member_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_cafe_gene_tree_member_by_id("homo_sapiens", "ENSG00000157764", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(
        req.path(),
        "/cafe/genetree/member/id/homo_sapiens/ENSG00000157764"
    );
}

#[test]
fn get_gene_tree_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_gene_tree_by_id("ENSGT00390000003602", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/genetree/id/ENSGT00390000003602");
}

#[test]
fn get_gene_tree_member_by_symbol() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_gene_tree_member_by_symbol("homo_sapiens", "BRCA2", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/genetree/member/symbol/homo_sapiens/BRCA2");
}

#[test]
fn get_gene_tree_member_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_gene_tree_member_by_id("homo_sapiens", "ENSG00000157764", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(
        req.path(),
        "/genetree/member/id/homo_sapiens/ENSG00000157764"
    );
}

#[test]
fn get_alignment_by_region() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_alignment_by_region("homo_sapiens", "X:1000000..1000100:1", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(
        req.path(),
        "/alignment/region/homo_sapiens/X:1000000..1000100:1"
    );
}

#[test]
fn get_homology_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_homology_by_id("homo_sapiens", "ENSG00000157764", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/homology/id/homo_sapiens/ENSG00000157764");
}

#[test]
fn get_homology_by_symbol() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_homology_by_symbol("homo_sapiens", "BRCA2", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/homology/symbol/homo_sapiens/BRCA2");
}

// ---- info ----

#[test]
fn get_info_analysis() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_info_analysis("homo_sapiens", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/analysis/homo_sapiens");
}

#[test]
fn get_info_assembly() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_info_assembly("homo_sapiens", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/assembly/homo_sapiens");
}

#[test]
fn get_info_assembly_region() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_info_assembly_region("homo_sapiens", "X", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/assembly/homo_sapiens/X");
}

#[test]
fn get_info_biotypes() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_info_biotypes("homo_sapiens", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/biotypes/homo_sapiens");
}

#[test]
fn get_info_biotypes_by_group() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_info_biotypes_by_group("coding", "gene", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/biotypes/groups/coding/gene");
}

#[test]
fn get_info_biotypes_by_name() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_info_biotypes_by_name("protein_coding", "gene", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/biotypes/name/protein_coding/gene");
}

#[test]
fn get_info_compara_methods() {
    let server = MockServer::with_json(200, "[]");
    client(&server).get_info_compara_methods(&[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/compara/methods");
}

#[test]
fn get_info_compara_species_sets() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_info_compara_species_sets("EPO", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/compara/species_sets/EPO");
}

#[test]
fn get_info_comparas() {
    let server = MockServer::with_json(200, "[]");
    client(&server).get_info_comparas(&[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/comparas");
}

#[test]
fn get_info_data() {
    let server = MockServer::with_json(200, "[]");
    client(&server).get_info_data(&[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/data");
}

#[test]
fn get_info_eg_version() {
    let server = MockServer::with_json(200, "{}");
    client(&server).get_info_eg_version(&[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/eg_version");
}

#[test]
fn get_info_external_dbs() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_info_external_dbs("homo_sapiens", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/external_dbs/homo_sapiens");
}

#[test]
fn get_info_divisions() {
    let server = MockServer::with_json(200, "[]");
    client(&server).get_info_divisions(&[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/divisions");
}

#[test]
fn get_info_genomes_by_name() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_info_genomes_by_name("homo_sapiens", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/genomes/homo_sapiens");
}

#[test]
fn get_info_genomes_by_accession() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_info_genomes_by_accession("GCA_000001405.28", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/genomes/accession/GCA_000001405.28");
}

#[test]
fn get_info_genomes_by_assembly() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_info_genomes_by_assembly("GRCh38", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/genomes/assembly/GRCh38");
}

#[test]
fn get_info_genomes_by_division() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_info_genomes_by_division("EnsemblVertebrates", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/genomes/division/EnsemblVertebrates");
}

#[test]
fn get_info_genomes_by_taxonomy() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_info_genomes_by_taxonomy("Homo_sapiens", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/genomes/taxonomy/Homo_sapiens");
}

#[test]
fn get_info_ping() {
    let server = MockServer::with_json(200, r#"{"ping":1}"#);
    client(&server).get_info_ping(&[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/ping");
}

#[test]
fn get_info_rest() {
    let server = MockServer::with_json(200, "{}");
    client(&server).get_info_rest(&[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/rest");
}

#[test]
fn get_info_software() {
    let server = MockServer::with_json(200, "{}");
    client(&server).get_info_software(&[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/software");
}

#[test]
fn get_info_species() {
    let server = MockServer::with_json(200, "{}");
    client(&server).get_info_species(&[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/species");
}

#[test]
fn get_info_variation_by_species() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_info_variation_by_species("homo_sapiens", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/variation/homo_sapiens");
}

#[test]
fn get_info_variation_consequence_types() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_info_variation_consequence_types(&[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/variation/consequence_types");
}

#[test]
fn get_info_variation_population_individuals() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_info_variation_population_individuals("homo_sapiens", "1000GENOMES:phase_3:ALL", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(
        req.path(),
        "/info/variation/populations/homo_sapiens/1000GENOMES:phase_3:ALL"
    );
}

#[test]
fn get_info_variation_populations() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_info_variation_populations("homo_sapiens", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/info/variation/populations/homo_sapiens");
}

// ---- ontology ----

#[test]
fn get_ancestors_by_id() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_ancestors_by_id("GO:0005667", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ontology/ancestors/GO:0005667");
}

#[test]
fn get_ancestors_chart_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_ancestors_chart_by_id("GO:0005667", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ontology/ancestors/chart/GO:0005667");
}

#[test]
fn get_descendants_by_id() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_descendants_by_id("GO:0005667", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ontology/descendants/GO:0005667");
}

#[test]
fn get_ontology_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_ontology_by_id("GO:0005667", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ontology/id/GO:0005667");
}

#[test]
fn get_ontology_by_name() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_ontology_by_name("transcription_factor_complex", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ontology/name/transcription_factor_complex");
}

#[test]
fn get_taxonomy_classification_by_id() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_taxonomy_classification_by_id("9606", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/taxonomy/classification/9606");
}

#[test]
fn get_taxonomy_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server).get_taxonomy_by_id("9606", &[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/taxonomy/id/9606");
}

#[test]
fn get_taxonomy_by_name() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_taxonomy_by_name("Homo_sapiens", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/taxonomy/name/Homo_sapiens");
}

// ---- ld ----

#[test]
fn get_ld_id() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_ld_id("homo_sapiens", "rs56116432", "1000GENOMES:phase_3:ALL", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(
        req.path(),
        "/ld/homo_sapiens/rs56116432/1000GENOMES:phase_3:ALL"
    );
}

#[test]
fn get_ld_pairwise() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_ld_pairwise("homo_sapiens", "rs6792369", "rs1042779", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ld/homo_sapiens/pairwise/rs6792369/rs1042779");
}

#[test]
fn get_ld_region() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_ld_region(
            "homo_sapiens",
            "6:25837556..25843455",
            "1000GENOMES:phase_3:ALL",
            &[],
        )
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(
        req.path(),
        "/ld/homo_sapiens/region/6:25837556..25843455/1000GENOMES:phase_3:ALL"
    );
}

// ---- regulation ----

#[test]
fn get_regulation_binding_matrix() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_regulation_binding_matrix("homo_sapiens", "MA0004.1", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/species/homo_sapiens/binding_matrix/MA0004.1/");
}

// ---- transcript ----

#[test]
fn get_transcript_haplotypes() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_transcript_haplotypes("homo_sapiens", "ENST00000288602", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(
        req.path(),
        "/transcript_haplotypes/homo_sapiens/ENST00000288602"
    );
}

// ---- ga4gh ----

#[test]
fn get_ga4gh_beacon() {
    let server = MockServer::with_json(200, "{}");
    client(&server).get_ga4gh_beacon(&[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ga4gh/beacon");
}

#[test]
fn get_ga4gh_beacon_query() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_ga4gh_beacon_query("A", "GRCh38", "C", "1", 100_176_903, &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ga4gh/beacon/query");
    assert!(req.query().contains("alternateBases=A"));
    assert!(req.query().contains("assemblyId=GRCh38"));
    assert!(req.query().contains("referenceBases=C"));
    assert!(req.query().contains("referenceName=1"));
    assert!(req.query().contains("start=100176903"));
}

#[test]
fn post_ga4gh_beacon_query() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .post_ga4gh_beacon_query(
            Some("A"),
            Some("GRCh38"),
            Some(100_176_904),
            Some("C"),
            Some("1"),
            Some(100_176_903),
            None,
            &[],
        )
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/ga4gh/beacon/query");
    assert_eq!(req.json()["alternateBases"], "A");
    assert_eq!(req.json()["assemblyId"], "GRCh38");
    assert_eq!(req.json()["end"], 100_176_904);
    assert_eq!(req.json()["referenceBases"], "C");
    assert_eq!(req.json()["referenceName"], "1");
    assert_eq!(req.json()["start"], 100_176_903);
    assert!(req.json().get("variantType").is_none());
}

#[test]
fn get_ga4gh_features_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_ga4gh_features_by_id("ENSG00000157764.7", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ga4gh/features/ENSG00000157764.7");
}

#[test]
fn search_ga4gh_features() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .search_ga4gh_features(
            Some(1_000_100),
            Some("1"),
            Some(1_000_000),
            Some("Ensembl"),
            None,
            &[],
        )
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/ga4gh/features/search");
    assert_eq!(req.json()["end"], 1_000_100);
    assert_eq!(req.json()["referenceName"], "1");
    assert_eq!(req.json()["start"], 1_000_000);
    assert_eq!(req.json()["featureSetId"], "Ensembl");
    assert!(req.json().get("parentId").is_none());
}

#[test]
fn search_ga4gh_callset() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .search_ga4gh_callset(Some("1"), Some("NA12878"), None, Some(10), &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/ga4gh/callsets/search");
    assert_eq!(req.json()["variantSetId"], "1");
    assert_eq!(req.json()["name"], "NA12878");
    assert_eq!(req.json()["pageSize"], 10);
    assert!(req.json().get("pageToken").is_none());
}

#[test]
fn get_ga4gh_callset_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server).get_ga4gh_callset_by_id("1", &[]).unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ga4gh/callsets/1");
}

#[test]
fn search_ga4gh_datasets() {
    // `page_token` is `Option<&serde_json::Value>` because Ensembl's docs
    // (Integer) and the GA4GH spec (string) disagree on its type; this
    // exercises the integer shape passed straight through.
    let page_token = json!(42);
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .search_ga4gh_datasets(Some(&page_token), Some(5), &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/ga4gh/datasets/search");
    assert_eq!(req.json()["pageToken"], 42);
    assert_eq!(req.json()["pageSize"], 5);
}

#[test]
fn get_ga4gh_datasets_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_ga4gh_datasets_by_id("6e340c4d1e333c7a676b1710d2e3953c", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(
        req.path(),
        "/ga4gh/datasets/6e340c4d1e333c7a676b1710d2e3953c"
    );
}

#[test]
fn search_ga4gh_featuresets() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .search_ga4gh_featuresets(Some("Ensembl"), None, Some(10), &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/ga4gh/featuresets/search");
    assert_eq!(req.json()["datasetId"], "Ensembl");
    assert_eq!(req.json()["pageSize"], 10);
    assert!(req.json().get("pageToken").is_none());
}

#[test]
fn get_ga4gh_featuresets_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_ga4gh_featuresets_by_id("Ensembl", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ga4gh/featuresets/Ensembl");
}

#[test]
fn get_ga4gh_variants_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_ga4gh_variants_by_id("1:rs1333049", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ga4gh/variants/1:rs1333049");
}

#[test]
fn search_ga4gh_variant_annotations() {
    // `effects` is documented as an array of `OntologyTerm` objects, not
    // bare strings, so it is `Option<&serde_json::Value>` and passed through
    // verbatim. This proves the passthrough works for a non-trivial value: a
    // real array-of-objects shape that `&[&str]` could never have expressed.
    let effects = json!([{"id": "SO:0001627", "term": "intron_variant"}]);
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .search_ga4gh_variant_annotations(
            Some("Ensembl"),
            Some(&effects),
            Some(1_000_100),
            Some(10),
            None,
            None,
            Some("1"),
            Some(1_000_000),
            &[],
        )
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/ga4gh/variantannotations/search");
    assert_eq!(req.json()["variantAnnotationSetId"], "Ensembl");
    assert_eq!(req.json()["effects"][0]["id"], "SO:0001627");
    assert_eq!(req.json()["effects"][0]["term"], "intron_variant");
    assert_eq!(req.json()["end"], 1_000_100);
    assert_eq!(req.json()["pageSize"], 10);
    assert_eq!(req.json()["referenceName"], "1");
    assert_eq!(req.json()["start"], 1_000_000);
    assert!(req.json().get("pageToken").is_none());
    assert!(req.json().get("referenceId").is_none());
}

#[test]
fn search_ga4gh_variants() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .search_ga4gh_variants(
            Some("1"),
            Some(&["NA12878"]),
            Some("1"),
            Some(1_000_000),
            Some(1_000_100),
            None,
            Some(10),
            &[],
        )
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/ga4gh/variants/search");
    assert_eq!(req.json()["variantSetId"], "1");
    assert_eq!(req.json()["callSetIds"][0], "NA12878");
    assert_eq!(req.json()["referenceName"], "1");
    assert_eq!(req.json()["start"], 1_000_000);
    assert_eq!(req.json()["end"], 1_000_100);
    assert_eq!(req.json()["pageSize"], 10);
    assert!(req.json().get("pageToken").is_none());
}

#[test]
fn search_ga4gh_variantsets() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .search_ga4gh_variantsets(Some("1"), None, Some(10), &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/ga4gh/variantsets/search");
    assert_eq!(req.json()["datasetId"], "1");
    assert_eq!(req.json()["pageSize"], 10);
    assert!(req.json().get("pageToken").is_none());
}

#[test]
fn get_ga4gh_variantsets_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_ga4gh_variantsets_by_id("1", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ga4gh/variantsets/1");
}

#[test]
fn search_ga4gh_references() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .search_ga4gh_references(
            Some("GRCh38"),
            None,
            Some("GCA_000001405"),
            None,
            Some(10),
            &[],
        )
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/ga4gh/references/search");
    assert_eq!(req.json()["referenceSetId"], "GRCh38");
    assert_eq!(req.json()["accession"], "GCA_000001405");
    assert_eq!(req.json()["pageSize"], 10);
    assert!(req.json().get("md5checksum").is_none());
    assert!(req.json().get("pageToken").is_none());
}

#[test]
fn get_ga4gh_references_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_ga4gh_references_by_id("GRCh38:1", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ga4gh/references/GRCh38:1");
}

#[test]
fn search_ga4gh_referencesets() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .search_ga4gh_referencesets(Some("GCA_000001405"), None, Some(10), &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/ga4gh/referencesets/search");
    assert_eq!(req.json()["accession"], "GCA_000001405");
    assert_eq!(req.json()["pageSize"], 10);
    assert!(req.json().get("pageToken").is_none());
}

#[test]
fn get_ga4gh_referencesets_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_ga4gh_referencesets_by_id("GRCh38", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ga4gh/referencesets/GRCh38");
}

#[test]
fn search_ga4gh_variant_annotationsets() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .search_ga4gh_variant_annotationsets(Some("1"), None, Some(10), &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/ga4gh/variantannotationsets/search");
    assert_eq!(req.json()["variantSetId"], "1");
    assert_eq!(req.json()["pageSize"], 10);
    assert!(req.json().get("pageToken").is_none());
}

#[test]
fn get_ga4gh_variant_annotationsets_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_ga4gh_variant_annotationsets_by_id("Ensembl", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/ga4gh/variantannotationsets/Ensembl");
}

// ---- variation ----

#[test]
fn get_variation_recoder_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_variation_recoder_by_id("human", "rs56116432", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/variant_recoder/human/rs56116432");
}

#[test]
fn get_variation_recoder_by_multiple_ids() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_variation_recoder_by_multiple_ids("human", &["rs56116432", "COSM476"], &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/variant_recoder/human");
    assert_eq!(req.json()["ids"][0], "rs56116432");
    assert_eq!(req.json()["ids"][1], "COSM476");
}

#[test]
fn get_variation_by_id() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_variation_by_id("homo_sapiens", "rs56116432", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/variation/homo_sapiens/rs56116432");
}

#[test]
fn get_variation_by_pmcid() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_variation_by_pmcid("homo_sapiens", "PMC5002951", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/variation/homo_sapiens/pmcid/PMC5002951");
}

#[test]
fn get_variation_by_pmid() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_variation_by_pmid("homo_sapiens", "26318936", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/variation/homo_sapiens/pmid/26318936");
}

#[test]
fn get_variation_by_multiple_ids() {
    let server = MockServer::with_json(200, "{}");
    client(&server)
        .get_variation_by_multiple_ids("homo_sapiens", &["rs56116432", "COSM476"], &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/variation/homo_sapiens");
    assert_eq!(req.json()["ids"][0], "rs56116432");
    assert_eq!(req.json()["ids"][1], "COSM476");
}

// ---- vep ----

#[test]
fn get_variant_consequences_by_hgvs_notation() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_variant_consequences_by_hgvs_notation("human", "AGT:c.803T>C", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    // `>` is not in the path-safe set, so it is percent-encoded.
    assert_eq!(req.path(), "/vep/human/hgvs/AGT:c.803T%3EC");
}

#[test]
fn get_variant_consequences_by_multiple_hgvs_notations() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_variant_consequences_by_multiple_hgvs_notations(
            "human",
            &["AGT:c.803T>C", "9:g.22125504G>C"],
            &[],
        )
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/vep/human/hgvs/");
    assert_eq!(req.json()["hgvs_notations"][0], "AGT:c.803T>C");
    assert_eq!(req.json()["hgvs_notations"][1], "9:g.22125504G>C");
}

#[test]
fn get_variant_consequences_by_id() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_variant_consequences_by_id("human", "rs56116432", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/vep/human/id/rs56116432");
}

#[test]
fn get_variant_consequences_by_multiple_ids() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_variant_consequences_by_multiple_ids("human", &["rs56116432", "COSM476"], &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/vep/human/id");
    assert_eq!(req.json()["ids"][0], "rs56116432");
    assert_eq!(req.json()["ids"][1], "COSM476");
}

#[test]
fn get_variant_consequences_by_region() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_variant_consequences_by_region("human", "9:22125503-22125502:1", "C", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/vep/human/region/9:22125503-22125502:1/C");
}

#[test]
fn get_variant_consequences_by_multiple_regions() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_variant_consequences_by_multiple_regions(
            "human",
            &["9 22125503 22125502 1/C . . ."],
            &[],
        )
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "POST");
    assert_eq!(req.path(), "/vep/human/region");
    assert_eq!(req.json()["variants"][0], "9 22125503 22125502 1/C . . .");
}

// ---- phenotype ----

#[test]
fn get_phenotype_by_accession() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_phenotype_by_accession("homo_sapiens", "EFO:0003877", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/phenotype/accession/homo_sapiens/EFO:0003877");
}

#[test]
fn get_phenotype_by_gene() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_phenotype_by_gene("homo_sapiens", "BRCA2", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/phenotype/gene/homo_sapiens/BRCA2");
}

#[test]
fn get_phenotype_by_region() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_phenotype_by_region("homo_sapiens", "9:22125500-22125502", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    assert_eq!(
        req.path(),
        "/phenotype/region/homo_sapiens/9:22125500-22125502"
    );
}

#[test]
fn get_phenotype_by_term() {
    let server = MockServer::with_json(200, "[]");
    client(&server)
        .get_phenotype_by_term("homo_sapiens", "coronary heart disease", &[])
        .unwrap();

    let req = server.only_request();
    assert_eq!(req.method, "GET");
    // Spaces in a path segment are percent-encoded.
    assert_eq!(
        req.path(),
        "/phenotype/term/homo_sapiens/coronary%20heart%20disease"
    );
}
