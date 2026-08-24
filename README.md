# ensemblrest-rs

A Rust client library for the [Ensembl REST API](https://rest.ensembl.org/).
`ensemblrest` is a port of [`goensemblrest`](https://github.com/gawbul/goensemblrest),
itself a port of [`pyEnsemblRest`](https://github.com/gawbul/pyEnsemblRest), and covers
all 106 Ensembl REST endpoints with typed methods, client-side sliding-window rate
limiting, and exponential-backoff retries for transient failures.

## Install

```bash
cargo add ensemblrest
```

## Quickstart

```rust,no_run
use ensemblrest::types::LookupRecord;
use ensemblrest::Client;

# fn main() -> ensemblrest::Result<()> {
let client = Client::new()?;
let braf: LookupRecord = client.get_lookup_by_id("ENSG00000157764", &[])?.json()?;
println!("{} is a {}", braf.display_name, braf.biotype);
# Ok(())
# }
```

## Decoding responses

Every endpoint method returns a `Response`. Decode it with `Response::json` for JSON,
or `Response::text` for the formats Ensembl serves as text.

```rust,no_run
use ensemblrest::options::content_type;
use ensemblrest::Client;

# fn main() -> ensemblrest::Result<()> {
let client = Client::new()?;
let fasta = client
    .get_sequence_by_id("ENSG00000157764", &[content_type("text/x-fasta")])?
    .text()?;
# Ok(())
# }
```

`json` accepts any `serde::Deserialize` type, so endpoints without a model in `types`
still work via `serde_json::Value`.

## Configuration

```rust,no_run
use ensemblrest::Client;
use std::time::Duration;

# fn main() -> ensemblrest::Result<()> {
let client = Client::builder()
    .timeout(Duration::from_secs(30))
    .rate_limit(15, Duration::from_secs(1))
    .max_attempts(5)
    .user_agent("my-tool/1.0")
    .max_response_bytes(256 * 1024 * 1024)
    .build()?;
# Ok(())
# }
```

`max_response_bytes` is the one builder option with no counterpart in the Go port.
`ureq` refuses response bodies over 10 MiB by default, which several Ensembl endpoints
exceed routinely -- large `overlap` result sets, multi-region `sequence` POSTs and
`alignment/region` blocks. This crate raises the default to 100 MiB
(`DEFAULT_MAX_RESPONSE_BYTES`); raise it further, as above, if you hit
`Error::Transport` on a large query, or lower it to bound memory use.

`Client` is cheap to clone and shares one rate limiter and connection pool across
clones, so a cloned client observes one global rate limit.

## Errors

```rust,no_run
use ensemblrest::{ApiErrorKind, Client};

# fn main() -> ensemblrest::Result<()> {
let client = Client::new()?;
match client.get_lookup_by_id("NOT_A_REAL_ID", &[]) {
    Ok(response) => println!("{}", response.status()),
    Err(e) if e.is_not_found() => println!("no such identifier"),
    Err(e) if e.api_kind() == Some(ApiErrorKind::RateLimit) => println!("rate limited"),
    Err(e) => println!("failed: {e}"),
}
# Ok(())
# }
```

## Dependencies

This crate has exactly three direct dependencies -- `ureq`, `serde` and `serde_json` --
and zero dev-dependencies. The mock HTTP server used in tests is hand-written on
`std`. This is a deliberate, defining constraint of the project: every addition to
either list is treated as a project failure, not a convenience.

## Development

Standard developer tasks are wired up in the `Makefile`:

| Target | Command | Purpose |
|---|---|---|
| `make all` | `lint test build` | Default: lint, test, then build |
| `make build` | `cargo build --all-targets` | Compiles the library, tests and examples |
| `make test` | `cargo test` | Runs the offline test suite |
| `make test-live` | `ENSEMBL_LIVE_TESTS=1 cargo test --test live -- --ignored` | Runs live smoke tests against `rest.ensembl.org` |
| `make test-coverage` | `cargo llvm-cov --html --open` | Generates an HTML coverage report (requires `cargo-llvm-cov`) |
| `make lint` | `cargo clippy --all-targets -- -D warnings` | Runs Clippy with warnings denied |
| `make format` | `cargo fmt` | Formats all source files |
| `make format-check` | `cargo fmt --check` | Checks formatting without writing |
| `make example` | `cargo run --example basic` | Runs the example application |
| `make clean` | `cargo clean` | Removes build artifacts |

There is deliberately no `test-race` target: Go's `-race` detector has no Rust
equivalent because `Send` and `Sync` are checked at compile time, so `make test`
already covers what it would catch.

See [AGENTS.md](AGENTS.md) for the full contributor and agent guide.

## Endpoint catalog

| Endpoint | Method | Path |
|---|---|---|
| `getArchiveById` | GET | `/archive/id/{{id}}` |
| `getArchiveByMultipleIds` | POST | `/archive/id` |
| `getCafeGeneTreeById` | GET | `/cafe/genetree/id/{{id}}` |
| `getCafeGeneTreeMemberBySymbol` | GET | `/cafe/genetree/member/symbol/{{species}}/{{symbol}}` |
| `getCafeGeneTreeMemberById` | GET | `/cafe/genetree/member/id/{{species}}/{{id}}` |
| `getGeneTreeById` | GET | `/genetree/id/{{id}}` |
| `getGeneTreeMemberBySymbol` | GET | `/genetree/member/symbol/{{species}}/{{symbol}}` |
| `getGeneTreeMemberById` | GET | `/genetree/member/id/{{species}}/{{id}}` |
| `getAlignmentByRegion` | GET | `/alignment/region/{{species}}/{{region}}` |
| `getHomologyById` | GET | `/homology/id/{{species}}/{{id}}` |
| `getHomologyBySymbol` | GET | `/homology/symbol/{{species}}/{{symbol}}` |
| `getXrefsBySymbol` | GET | `/xrefs/symbol/{{species}}/{{symbol}}` |
| `getXrefsById` | GET | `/xrefs/id/{{id}}` |
| `getXrefsByName` | GET | `/xrefs/name/{{species}}/{{name}}` |
| `getInfoAnalysis` | GET | `/info/analysis/{{species}}` |
| `getInfoAssembly` | GET | `/info/assembly/{{species}}` |
| `getInfoAssemblyRegion` | GET | `/info/assembly/{{species}}/{{region_name}}` |
| `getInfoBiotypes` | GET | `/info/biotypes/{{species}}` |
| `getInfoBiotypesByGroup` | GET | `/info/biotypes/groups/{{group}}/{{object_type}}` |
| `getInfoBiotypesByName` | GET | `/info/biotypes/name/{{name}}/{{object_type}}` |
| `getInfoComparaMethods` | GET | `/info/compara/methods` |
| `getInfoComparaSpeciesSets` | GET | `/info/compara/species_sets/{{methods}}` |
| `getInfoComparas` | GET | `/info/comparas` |
| `getInfoData` | GET | `/info/data` |
| `getInfoEgVersion` | GET | `/info/eg_version` |
| `getInfoExternalDbs` | GET | `/info/external_dbs/{{species}}` |
| `getInfoDivisions` | GET | `/info/divisions` |
| `getInfoGenomesByName` | GET | `/info/genomes/{{name}}` |
| `getInfoGenomesByAccession` | GET | `/info/genomes/accession/{{accession}}` |
| `getInfoGenomesByAssembly` | GET | `/info/genomes/assembly/{{assembly_id}}` |
| `getInfoGenomesByDivision` | GET | `/info/genomes/division/{{division}}` |
| `getInfoGenomesByTaxonomy` | GET | `/info/genomes/taxonomy/{{taxon_name}}` |
| `getInfoPing` | GET | `/info/ping` |
| `getInfoRest` | GET | `/info/rest` |
| `getInfoSoftware` | GET | `/info/software` |
| `getInfoSpecies` | GET | `/info/species` |
| `getInfoVariationBySpecies` | GET | `/info/variation/{{species}}` |
| `getInfoVariationConsequenceTypes` | GET | `/info/variation/consequence_types` |
| `getInfoVariationPopulationIndividuals` | GET | `/info/variation/populations/{{species}}/{{population_name}}` |
| `getInfoVariationPopulations` | GET | `/info/variation/populations/{{species}}` |
| `getLdId` | GET | `/ld/{{species}}/{{id}}/{{population_name}}` |
| `getLdPairwise` | GET | `/ld/{{species}}/pairwise/{{id1}}/{{id2}}` |
| `getLdRegion` | GET | `/ld/{{species}}/region/{{region}}/{{population_name}}` |
| `getLookupById` | GET | `/lookup/id/{{id}}` |
| `getLookupByMultipleIds` | POST | `/lookup/id` |
| `getLookupBySymbol` | GET | `/lookup/symbol/{{species}}/{{symbol}}` |
| `getLookupByMultipleSymbols` | POST | `/lookup/symbol/{{species}}` |
| `getMapCdnaToRegion` | GET | `/map/cdna/{{id}}/{{region}}` |
| `getMapCdsToRegion` | GET | `/map/cds/{{id}}/{{region}}` |
| `getMapAssemblyOneToTwo` | GET | `/map/{{species}}/{{asm_one}}/{{region}}/{{asm_two}}` |
| `getMapTranslationToRegion` | GET | `/map/translation/{{id}}/{{region}}` |
| `getAncestorsById` | GET | `/ontology/ancestors/{{id}}` |
| `getAncestorsChartById` | GET | `/ontology/ancestors/chart/{{id}}` |
| `getDescendantsById` | GET | `/ontology/descendants/{{id}}` |
| `getOntologyById` | GET | `/ontology/id/{{id}}` |
| `getOntologyByName` | GET | `/ontology/name/{{name}}` |
| `getTaxonomyClassificationById` | GET | `/taxonomy/classification/{{id}}` |
| `getTaxonomyById` | GET | `/taxonomy/id/{{id}}` |
| `getTaxonomyByName` | GET | `/taxonomy/name/{{name}}` |
| `getOverlapById` | GET | `/overlap/id/{{id}}` |
| `getOverlapByRegion` | GET | `/overlap/region/{{species}}/{{region}}` |
| `getOverlapByTranslation` | GET | `/overlap/translation/{{id}}` |
| `getPhenotypeByAccession` | GET | `/phenotype/accession/{{species}}/{{accession}}` |
| `getPhenotypeByGene` | GET | `/phenotype/gene/{{species}}/{{gene}}` |
| `getPhenotypeByRegion` | GET | `/phenotype/region/{{species}}/{{region}}` |
| `getPhenotypeByTerm` | GET | `/phenotype/term/{{species}}/{{term}}` |
| `getRegulationBindingMatrix` | GET | `/species/{{species}}/binding_matrix/{{binding_matrix}}/` |
| `getSequenceById` | GET | `/sequence/id/{{id}}` |
| `getSequenceByMultipleIds` | POST | `/sequence/id` |
| `getSequenceByRegion` | GET | `/sequence/region/{{species}}/{{region}}` |
| `getSequenceByMultipleRegions` | POST | `/sequence/region/{{species}}` |
| `getTranscriptHaplotypes` | GET | `/transcript_haplotypes/{{species}}/{{id}}` |
| `getVariantConsequencesByHGVSNotation` | GET | `/vep/{{species}}/hgvs/{{hgvs_notation}}` |
| `getVariantConsequencesByMultipleHGVSNotations` | POST | `/vep/{{species}}/hgvs/` |
| `getVariantConsequencesById` | GET | `/vep/{{species}}/id/{{id}}` |
| `getVariantConsequencesByMultipleIds` | POST | `/vep/{{species}}/id` |
| `getVariantConsequencesByRegion` | GET | `/vep/{{species}}/region/{{region}}/{{allele}}` |
| `getVariantConsequencesByMultipleRegions` | POST | `/vep/{{species}}/region` |
| `getVariationRecoderById` | GET | `/variant_recoder/{{species}}/{{id}}` |
| `getVariationRecoderByMultipleIds` | POST | `/variant_recoder/{{species}}` |
| `getVariationById` | GET | `/variation/{{species}}/{{id}}` |
| `getVariationByPMCID` | GET | `/variation/{{species}}/pmcid/{{pmcid}}` |
| `getVariationByPMID` | GET | `/variation/{{species}}/pmid/{{pmid}}` |
| `getVariationByMultipleIds` | POST | `/variation/{{species}}` |
| `getGA4GHBeacon` | GET | `/ga4gh/beacon` |
| `getGA4GHBeaconQuery` | GET | `/ga4gh/beacon/query` |
| `postGA4GHBeaconQuery` | POST | `/ga4gh/beacon/query` |
| `getGA4GHFeaturesById` | GET | `/ga4gh/features/{{id}}` |
| `searchGA4GHFeatures` | POST | `/ga4gh/features/search` |
| `searchGA4GHCallset` | POST | `/ga4gh/callsets/search` |
| `getGA4GHCallsetById` | GET | `/ga4gh/callsets/{{id}}` |
| `searchGA4GHDatasets` | POST | `/ga4gh/datasets/search` |
| `getGA4GHDatasetsById` | GET | `/ga4gh/datasets/{{id}}` |
| `searchGA4GHFeaturesets` | POST | `/ga4gh/featuresets/search` |
| `getGA4GHFeaturesetsById` | GET | `/ga4gh/featuresets/{{id}}` |
| `getGA4GHVariantsById` | GET | `/ga4gh/variants/{{id}}` |
| `searchGA4GHVariantAnnotations` | POST | `/ga4gh/variantannotations/search` |
| `searchGA4GHVariants` | POST | `/ga4gh/variants/search` |
| `searchGA4GHVariantsets` | POST | `/ga4gh/variantsets/search` |
| `getGA4GHVariantsetsById` | GET | `/ga4gh/variantsets/{{id}}` |
| `searchGA4GHReferences` | POST | `/ga4gh/references/search` |
| `getGA4GHReferencesById` | GET | `/ga4gh/references/{{id}}` |
| `searchGA4GHReferencesets` | POST | `/ga4gh/referencesets/search` |
| `getGA4GHReferencesetsById` | GET | `/ga4gh/referencesets/{{id}}` |
| `searchGA4GHVariantAnnotationsets` | POST | `/ga4gh/variantannotationsets/search` |
| `getGA4GHVariantAnnotationsetsById` | GET | `/ga4gh/variantannotationsets/{{id}}` |

## License

MIT. See [LICENSE](LICENSE).
