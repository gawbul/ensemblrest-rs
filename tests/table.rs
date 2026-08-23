//! Integration tests for the endpoint catalog and dynamic dispatch.

use ensemblrest::endpoints::{ENDPOINTS, Method, endpoint, endpoints};
use ensemblrest::serde_json;
use ensemblrest::{Client, Error};

mod common;
use common::mock::MockServer;

#[test]
fn the_table_has_exactly_the_endpoints_the_go_port_has() {
    assert_eq!(
        ENDPOINTS.len(),
        106,
        "the Go port's EndpointsTable has 106 entries"
    );
}

#[test]
fn every_name_is_unique() {
    let mut names: Vec<&str> = ENDPOINTS.iter().map(|e| e.name).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(names.len(), before, "duplicate endpoint name in ENDPOINTS");
}

#[test]
fn names_are_camel_case_not_snake_case() {
    // The table keys are the cross-port contract shared with goensemblrest
    // and pyEnsemblRest. Re-casing them silently breaks Client::call for
    // anyone porting code between the three libraries.
    //
    // goensemblrest's EndpointsTable genuinely has ten `search*`-prefixed GA4GH
    // endpoints (e.g. "searchGA4GHFeatures"), alongside the `get*`/`post*`
    // majority, so "search" is a third valid prefix here.
    for e in ENDPOINTS {
        assert!(!e.name.contains('_'), "{} must stay camelCase", e.name);
        assert!(
            e.name.starts_with("get") || e.name.starts_with("post") || e.name.starts_with("search"),
            "{} has an unexpected prefix",
            e.name
        );
    }
}

#[test]
fn every_url_is_rooted_and_every_entry_is_documented() {
    for e in ENDPOINTS {
        assert!(
            e.url.starts_with('/'),
            "{} url must start with '/': {}",
            e.name,
            e.url
        );
        assert!(!e.doc.is_empty(), "{} must have documentation", e.name);
        assert_eq!(e.content_type, "application/json", "{}", e.name);
    }
}

#[test]
fn only_post_endpoints_declare_post_parameters() {
    for e in ENDPOINTS {
        if e.method == Method::Get {
            assert!(
                e.post_parameters.is_empty(),
                "{} is GET but declares body params",
                e.name
            );
        }
    }
    let posts = ENDPOINTS
        .iter()
        .filter(|e| e.method == Method::Post)
        .count();
    assert_eq!(posts, 21, "the Go port has 21 POST endpoints");
}

#[test]
fn placeholders_in_urls_are_well_formed() {
    for e in ENDPOINTS {
        assert_eq!(
            e.url.matches("{{").count(),
            e.url.matches("}}").count(),
            "{} has unbalanced placeholders: {}",
            e.name,
            e.url
        );
    }
}

#[test]
fn lookup_by_name_works_and_is_consistent_with_the_slice() {
    let spec = endpoint("getLookupById").expect("getLookupById must exist");
    assert_eq!(spec.url, "/lookup/id/{{id}}");
    assert_eq!(spec.method, Method::Get);
    assert_eq!(endpoints().len(), ENDPOINTS.len());
    assert!(endpoint("noSuchEndpoint").is_none());
}

#[test]
fn spot_check_against_the_go_table() {
    // One of each shape, transcribed by hand from goensemblrest/endpoints.go.
    let archive_post = endpoint("getArchiveByMultipleIds").unwrap();
    assert_eq!(archive_post.url, "/archive/id");
    assert_eq!(archive_post.method, Method::Post);
    assert_eq!(
        archive_post.post_parameters,
        &["id"],
        "archive uses 'id', not 'ids'"
    );

    let lookup_post = endpoint("getLookupByMultipleIds").unwrap();
    assert_eq!(
        lookup_post.post_parameters,
        &["ids"],
        "lookup uses 'ids', not 'id'"
    );

    let two_params = endpoint("getHomologyBySymbol").unwrap();
    assert_eq!(two_params.url, "/homology/symbol/{{species}}/{{symbol}}");

    let no_params = endpoint("getGA4GHBeacon").unwrap();
    assert_eq!(no_params.url, "/ga4gh/beacon");
    assert_eq!(no_params.method, Method::Get);
}

#[test]
fn call_dispatches_by_name() {
    let server = MockServer::with_json(200, r#"{"id":"ENSG00000157764"}"#);
    let c = Client::builder()
        .base_url(server.base_url())
        .build()
        .unwrap();
    let v: serde_json::Value = c
        .call("getLookupById", &[("id", "ENSG00000157764")], None, &[])
        .unwrap()
        .json()
        .unwrap();

    assert_eq!(v["id"], "ENSG00000157764");
    assert_eq!(server.only_request().path(), "/lookup/id/ENSG00000157764");
}

#[test]
fn call_with_an_unknown_name_is_an_error_before_any_request() {
    let server = MockServer::with_json(200, "{}");
    let c = Client::builder()
        .base_url(server.base_url())
        .build()
        .unwrap();
    let err = c.call("noSuchEndpoint", &[], None, &[]).unwrap_err();

    assert!(
        matches!(&err, Error::UnknownEndpoint(n) if n == "noSuchEndpoint"),
        "got {err:?}"
    );
    assert_eq!(server.request_count(), 0);
}
