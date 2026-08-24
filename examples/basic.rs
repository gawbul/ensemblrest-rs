//! A tour of the `ensemblrest` client.
//!
//! Run with: `cargo run --example basic`

use ensemblrest::options::{content_type, query};
use ensemblrest::serde_json;
use ensemblrest::types::{LookupRecord, PingResponse, SpeciesResponse};
use ensemblrest::{ApiErrorKind, Client};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("ensemblrest-example/1.0")
        .build()?;

    let ping: PingResponse = client.get_info_ping(&[])?.json()?;
    println!("service up: {}", ping.ping == 1);

    let braf: LookupRecord = client.get_lookup_by_id("ENSG00000157764", &[])?.json()?;
    println!(
        "{} ({}) on {}:{}-{}",
        braf.display_name, braf.biotype, braf.seq_region_name, braf.start, braf.end
    );

    // Expanded lookups return nested transcripts.
    let expanded: LookupRecord = client
        .get_lookup_by_id("ENSG00000157764", &[query("expand", "1")])?
        .json()?;
    println!("transcripts: {}", expanded.transcripts.len());

    // Non-JSON content types come back through .text().
    let fasta = client
        .get_sequence_by_region(
            "homo_sapiens",
            "X:1000000..1000100:1",
            &[content_type("text/x-fasta")],
        )?
        .text()?;
    println!("fasta header: {}", fasta.lines().next().unwrap_or_default());

    // POST endpoints take slices.
    let records: Vec<LookupRecord> = client
        .get_archive_by_multiple_ids(&["ENSG00000157764", "ENSG00000248378"], &[])?
        .json()?;
    println!("archive records: {}", records.len());

    let species: SpeciesResponse = client.get_info_species(&[])?.json()?;
    println!("species known: {}", species.species.len());

    // Errors carry the status and the server's message.
    match client.get_lookup_by_id("NOT_A_REAL_ID", &[]) {
        Err(e) if e.api_kind() == Some(ApiErrorKind::BadRequest) => {
            println!("expected failure: {e}");
        }
        Err(e) => println!("unexpected failure: {e}"),
        Ok(_) => println!("unexpectedly succeeded"),
    }

    // Dynamic dispatch by endpoint name, for parity with the Go and Python ports.
    let v: serde_json::Value = client
        .call("getLookupById", &[("id", "ENSG00000157764")], None, &[])?
        .json()?;
    println!("dynamic dispatch: {}", v["display_name"]);

    if let Some(remaining) = client.rate_limit().remaining {
        println!("requests remaining: {remaining}");
    }

    Ok(())
}
