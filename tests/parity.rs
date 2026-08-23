//! Enforces the 106-endpoint/106-method invariant.

use ensemblrest::endpoints::ENDPOINTS;
use std::collections::BTreeMap;
use std::fs;

/// Modules that define typed endpoint methods.
///
/// `endpoints.rs` is excluded because it defines the table and the generic
/// `call`, not typed methods.
fn domain_sources() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for entry in fs::read_dir("src").expect("read src/") {
        let path = entry.expect("dir entry").path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        if matches!(
            name.as_str(),
            "lib.rs"
                | "endpoints.rs"
                | "client.rs"
                | "request.rs"
                | "response.rs"
                | "error.rs"
                | "options.rs"
                | "ratelimit.rs"
                | "encoding.rs"
                | "types.rs"
        ) {
            continue;
        }
        out.push((name, fs::read_to_string(&path).expect("read source")));
    }
    out
}

/// Extracts every endpoint name passed to `self.call("...")`.
fn called_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = source;
    while let Some(idx) = rest.find("self.call(") {
        // Check if this occurrence is on a commented line
        if let Some(line_start) = rest[..idx].rfind('\n') {
            let line = &rest[line_start + 1..idx];
            if line.trim_start().starts_with("//") {
                rest = &rest[idx + 1..];
                continue;
            }
        } else if rest[..idx].trim_start().starts_with("//") {
            rest = &rest[idx + 1..];
            continue;
        }
        rest = &rest[idx + "self.call(".len()..];
        // Skip whitespace and find the opening quote of the first string argument
        let trimmed = rest.trim_start();
        if let Some(stripped) = trimmed.strip_prefix('"') {
            if let Some(end) = stripped.find('"') {
                names.push(stripped[..end].to_string());
                rest = &stripped[end + 1..];
            } else {
                break;
            }
        } else {
            break;
        }
    }
    names
}

#[test]
fn there_are_sixteen_domain_modules() {
    let mut modules: Vec<String> = domain_sources().into_iter().map(|(n, _)| n).collect();
    modules.sort();
    assert_eq!(
        modules.len(),
        16,
        "expected 16 domain modules, found {modules:?}"
    );
}

#[test]
fn every_endpoint_is_reached_by_exactly_one_typed_method() {
    let mut counts: BTreeMap<&str, usize> = ENDPOINTS.iter().map(|e| (e.name, 0)).collect();

    for (_, source) in domain_sources() {
        for name in called_names(&source) {
            if let Some(count) = counts.get_mut(name.as_str()) {
                *count += 1;
            }
        }
    }

    let missing: Vec<&str> = counts
        .iter()
        .filter(|(_, c)| **c == 0)
        .map(|(n, _)| *n)
        .collect();
    assert!(
        missing.is_empty(),
        "endpoints with no typed method: {missing:?}"
    );

    let duplicated: Vec<&str> = counts
        .iter()
        .filter(|(_, c)| **c > 1)
        .map(|(n, _)| *n)
        .collect();
    assert!(
        duplicated.is_empty(),
        "endpoints with more than one method: {duplicated:?}"
    );
}

#[test]
fn every_typed_method_targets_an_endpoint_that_exists() {
    let known: Vec<&str> = ENDPOINTS.iter().map(|e| e.name).collect();
    let mut unknown = Vec::new();

    for (module, source) in domain_sources() {
        for name in called_names(&source) {
            if !known.contains(&name.as_str()) {
                unknown.push(format!("{module}: {name}"));
            }
        }
    }

    assert!(
        unknown.is_empty(),
        "methods calling names absent from ENDPOINTS: {unknown:?}"
    );
}

#[test]
fn the_method_count_matches_the_endpoint_count() {
    let total: usize = domain_sources()
        .iter()
        .map(|(_, s)| called_names(s).len())
        .sum();
    assert_eq!(
        total,
        ENDPOINTS.len(),
        "106 endpoints require 106 typed methods"
    );
}
