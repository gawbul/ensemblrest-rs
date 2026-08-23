//! Offline tests for every typed endpoint method.
//!
//! Each test asserts the HTTP method, resolved path and request body that a
//! typed method produces, against the std-only mock server.

mod common;

use common::mock::MockServer;
use ensemblrest::Client;

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
