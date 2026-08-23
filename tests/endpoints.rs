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
