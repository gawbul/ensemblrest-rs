//! Smoke tests against the live Ensembl REST API.
//!
//! These are `#[ignore]`d and additionally gated on `ENSEMBL_LIVE_TESTS=1`, so
//! neither `cargo test` nor `cargo test -- --ignored` touches the network
//! unless asked. Run with:
//!
//! ```text
//! ENSEMBL_LIVE_TESTS=1 cargo test --test live -- --ignored
//! ```

use ensemblrest::Client;
use ensemblrest::options::content_type;
use ensemblrest::types::{LookupRecord, PingResponse, SpeciesResponse};

/// Returns a live client, or `None` when live testing is not enabled.
fn live_client() -> Option<Client> {
    if std::env::var("ENSEMBL_LIVE_TESTS").as_deref() != Ok("1") {
        eprintln!("skipping: set ENSEMBL_LIVE_TESTS=1 to run live tests");
        return None;
    }
    Some(Client::new().expect("build client"))
}

#[test]
#[ignore = "requires network access and ENSEMBL_LIVE_TESTS=1"]
fn ping_reports_the_service_is_up() {
    let Some(c) = live_client() else { return };
    let ping: PingResponse = c.get_info_ping(&[]).unwrap().json().unwrap();
    assert_eq!(ping.ping, 1);
}

#[test]
#[ignore = "requires network access and ENSEMBL_LIVE_TESTS=1"]
fn lookup_returns_a_known_stable_gene() {
    let Some(c) = live_client() else { return };
    // BRAF is stable reference data and safe to assert on.
    let rec: LookupRecord = c
        .get_lookup_by_id("ENSG00000157764", &[])
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(rec.id, "ENSG00000157764");
    assert_eq!(rec.display_name, "BRAF");
    assert_eq!(rec.species, "homo_sapiens");
}

#[test]
#[ignore = "requires network access and ENSEMBL_LIVE_TESTS=1"]
fn species_list_includes_humans() {
    let Some(c) = live_client() else { return };
    let species: SpeciesResponse = c.get_info_species(&[]).unwrap().json().unwrap();
    assert!(species.species.iter().any(|s| s.name == "homo_sapiens"));
}

#[test]
#[ignore = "requires network access and ENSEMBL_LIVE_TESTS=1"]
fn sequence_can_be_fetched_as_fasta() {
    let Some(c) = live_client() else { return };
    let fasta = c
        .get_sequence_by_region(
            "homo_sapiens",
            "X:1000000..1000100:1",
            &[content_type("text/x-fasta")],
        )
        .unwrap()
        .text()
        .unwrap();
    assert!(
        fasta.starts_with('>'),
        "expected FASTA, got: {}",
        // Truncating by bytes would panic mid-UTF-8 on a non-ASCII error page.
        fasta.chars().take(80).collect::<String>()
    );
}

#[test]
#[ignore = "requires network access and ENSEMBL_LIVE_TESTS=1"]
fn an_unknown_identifier_is_a_clean_error() {
    let Some(c) = live_client() else { return };
    let err = c.get_lookup_by_id("ENSG00000000000", &[]).unwrap_err();
    assert!(err.is_bad_request() || err.is_not_found(), "got {err}");
}

#[test]
#[ignore = "requires network access and ENSEMBL_LIVE_TESTS=1"]
fn rate_limit_telemetry_is_reported() {
    let Some(c) = live_client() else { return };
    c.get_info_ping(&[]).unwrap();
    assert!(
        c.rate_limit().limit.is_some(),
        "Ensembl sends X-RateLimit-Limit"
    );
}
