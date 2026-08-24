# AGENTS.md

> Comprehensive agent and developer guide for **`ensemblrest`**.

---

## 1. Project overview

`ensemblrest` is an idiomatic Rust client library for the
[Ensembl REST API](https://rest.ensembl.org/). It is a port of
[`goensemblrest`](https://github.com/gawbul/goensemblrest), itself a port of
[`pyEnsemblRest`](https://github.com/gawbul/pyEnsemblRest), and provides
endpoint-for-endpoint parity across all three: 106 endpoints, client-side
sliding-window rate limiting, exponential-backoff retries for transient
failures, and idiomatic `Result`/`Error` error handling.

### Key characteristics

- **Crate name**: `ensemblrest`
- **Edition**: 2024, MSRV 1.98
- **Exactly three direct dependencies**: `ureq`, `serde`, `serde_json`, and
  **zero dev-dependencies** — see [Rule 1](#rule-1-dependency-budget) below.
- **Thread safety**: `Client` is `Send + Sync` and cheap to clone; clones share
  one rate limiter and connection pool.
- **License**: MIT.

---

## 2. Repository layout

```
.
├── Makefile                 # Developer automation targets
├── Cargo.toml                # Exactly 3 dependencies, 0 dev-dependencies
├── LICENSE                   # MIT License
├── README.md                  # Public docs, quickstart, endpoint catalog
├── AGENTS.md                  # This file
├── CLAUDE.md                  # Pointer to this file
│
├── src/
│   ├── lib.rs                 # Crate root, crate-level docs, re-exports
│   ├── client.rs               # Client / ClientBuilder
│   ├── request.rs              # HTTP execution, path resolution, retry loop
│   ├── ratelimit.rs             # Sliding-window rate limiter
│   ├── error.rs                 # Error, ApiError, ApiErrorKind
│   ├── endpoints.rs              # EndpointSpec, ENDPOINTS table, call_raw
│   ├── response.rs                # Response, .json() / .text() decoding
│   ├── options.rs                  # RequestOption (query/content_type/header)
│   ├── encoding.rs                  # Path-template resolution and encoding
│   ├── types.rs                      # Domain data models, null_to_default helper
│   │
│   ├── archive.rs  comparative.rs  ga4gh.rs  info.rs  ld.rs  lookup.rs
│   ├── mapping.rs  ontology.rs  overlap.rs  phenotype.rs  regulation.rs
│   ├── sequence.rs  transcript.rs  variation.rs  vep.rs  xrefs.rs
│   │                                 # 16 domain modules, each defining typed
│   │                                 # methods on `Client`
│
├── examples/
│   └── basic.rs                # Runnable example
│
└── tests/
    ├── common/                  # Shared test helpers, incl. hand-written mock server
    ├── endpoints.rs               # Offline mock-server tests, one per endpoint
    ├── parity.rs                   # Enforces the 106<->106 ENDPOINTS/method invariant
    ├── retry.rs                     # Retry/backoff behaviour
    ├── table.rs                      # ENDPOINTS table structural checks
    ├── mock_server.rs                 # Mock server self-tests
    └── live.rs                         # Live smoke tests against rest.ensembl.org,
                                          # #[ignore] and gated by ENSEMBL_LIVE_TESTS=1
```

---

## 3. Core architecture

### 3.1 Client configuration (builder pattern)

```rust
use ensemblrest::Client;
use std::time::Duration;

let client = Client::builder()
    .base_url("https://rest.ensembl.org")
    .timeout(Duration::from_secs(30))
    .rate_limit(15, Duration::from_secs(1))
    .max_attempts(5)
    .user_agent("my-tool/1.0")
    .header("X-Custom", "value")
    .build()?;
# Ok::<(), ensemblrest::Error>(())
```

`Client::new()` builds a client with all defaults.

### 3.2 Sliding-window rate limiting (`src/ratelimit.rs`)

- In-memory timestamp sliding window, protected by a `Mutex`.
- Default: 15 requests per 1-second window, matching Ensembl's server-side limit.
- Parses `X-RateLimit-*` and `Retry-After` response headers into `RateLimitInfo`,
  retrievable via `Client::rate_limit()`.

### 3.3 Request execution and retries (`src/request.rs`, `src/encoding.rs`)

- Path templates use `{{param}}` placeholders, resolved and percent-encoded —
  see [Rule 3](#rule-3-colon-preservation) for the one deliberate exception.
- Retries transient conditions (HTTP 408/429/500/502/503/504, and known Ensembl
  transient body strings) up to `max_attempts` times, honouring `Retry-After`
  when present, otherwise backing off as `attempt * wall_time * 2`.

### 3.4 Error handling (`src/error.rs`)

`Error` is the crate's error enum; `ApiError` carries the HTTP status, message,
rate-limit telemetry and raw body. `ApiErrorKind` classifies an `ApiError` by
status code (`BadRequest`, `NotFound`, `Timeout`, `RateLimit`, `InternalServer`,
`ServiceUnavailable`, `Other(u16)`). Convenience predicates (`is_not_found()`,
etc.) and `Error::api_kind()` cover the common cases without matching on the
enum directly.

### 3.5 Dual dispatch model (`src/endpoints.rs`)

1. **Typed domain methods** — one per endpoint, across the 16 domain modules
   (e.g. `client.get_lookup_by_id(id, opts)`).
2. **Dynamic dispatch** — `client.call_raw(method, path_template, path_params, body, opts)`
   for ad-hoc paths.
3. **Catalog introspection** — `ensemblrest::endpoints::ENDPOINTS` is the full,
   `'static` table of `EndpointSpec`s (name, doc, url, method, content_type,
   post_parameters).

### 3.6 Request options (`src/options.rs`)

Per-call customization via a `&[RequestOption]` slice, built with the `query`,
`content_type` and `header` free functions.

---

## 4. Development workflow

All standard tasks are wired up in the `Makefile`:

| Target | Command | Purpose |
|---|---|---|
| `make all` | `lint test build` | Default: lint, test, build |
| `make build` | `cargo build --all-targets` | Builds the library, tests and examples |
| `make test` | `cargo test` | Runs the offline test suite |
| `make test-live` | `ENSEMBL_LIVE_TESTS=1 cargo test --test live -- --ignored` | Live smoke tests against `rest.ensembl.org` |
| `make test-coverage` | `cargo llvm-cov --html --open` | HTML coverage report (requires `cargo-llvm-cov`) |
| `make lint` | `cargo clippy --all-targets -- -D warnings` | Clippy, warnings denied |
| `make format` | `cargo fmt` | Formats all source files |
| `make format-check` | `cargo fmt --check` | Checks formatting without writing |
| `make example` | `cargo run --example basic` | Runs the example application |
| `make clean` | `cargo clean` | Removes build artifacts |

There is deliberately **no `test-race` target**. Go's `-race` detector has no
Rust equivalent because `Send`/`Sync` are checked at compile time by the
borrow checker, not caught at runtime — `make test` already covers what it
would catch. Do not add one.

---

## 5. Hard-won rules

These are not stylistic preferences. Each one was a real defect or near-miss
during this project's build, and violating any of them is a project failure.

### Rule 1: Dependency budget

The crate has **exactly three** direct dependencies — `ureq`, `serde`,
`serde_json` — and **zero** dev-dependencies. The mock HTTP server used across
the test suite is hand-written on `std`, not pulled from a crate. Adding a
dependency or a dev-dependency, for any reason, is a project failure. If a
task seems to need one, stop and raise it instead of adding it.

### Rule 2: Endpoint keys stay camelCase

Names in `ENDPOINTS` (e.g. `getLookupById`) are the cross-port contract shared
verbatim with `goensemblrest` and `pyEnsemblRest`. They must never be
re-cased to snake_case or any other convention — even though the generated
Rust method names around them (`get_lookup_by_id`) are idiomatic snake_case.
The string key and the Rust method name are two different things; only the
key is the shared contract.

### Rule 3: Colon preservation

Path-template resolution must never percent-encode a colon. Genomic region
identifiers such as `13:32889611..32973805:1` depend on this — encoding the
colons to `%3A` breaks every genomic-region endpoint. This is enforced in
`src/encoding.rs`; if you touch path resolution, add a regression test that
asserts a colon survives resolution literally.

### Rule 4: Adding or changing an endpoint

To add a new endpoint (or modify an existing one):

1. Add its `EndpointSpec` to `ENDPOINTS` in `src/endpoints.rs` (name, doc,
   url with `{{param}}` placeholders, method, content_type, post_parameters).
2. Add the typed method on `Client` in the relevant domain module (or a new
   one, following the existing 16-module layout).
3. Add a test in `tests/endpoints.rs` covering the new method against the
   mock server.
4. Run `cargo test --test parity`. This enforces the 106<->106 invariant
   between `ENDPOINTS` and typed methods and will fail loudly if the two
   drift out of sync — do not skip it.

### Rule 5: `null` tolerance in models

Go's `encoding/json` silently accepts a JSON `null` into a `string` field.
Rust's `serde` does not, and `#[serde(default)]` alone does **not** fix this:
`default` only covers a **missing** key, not a key whose value is explicitly
`null`. Every non-`Option` field that Ensembl can return as `null` must use
`#[serde(default, deserialize_with = "null_to_default")]`, using the helper
defined in `src/types.rs`. This bug reached the live test suite before it was
caught — treat any new model field pulled from a live Ensembl response as
suspect until proven it can't be null.

### Rule 6: Reaching `serde_json` from outside the crate

Integration tests, examples and doctests cannot see the library's own regular
dependencies — Cargo does not expose a library's `[dependencies]` to its own
`tests/`, `examples/`, or doctests. Reach `serde_json` through the crate's own
re-export, `ensemblrest::serde_json`, rather than adding `serde_json` as a
dev-dependency (which would violate Rule 1).

---

## 6. Coding standards

1. Every public item must be documented; `#[warn(missing_docs)]` is on by
   default via `Cargo.toml`'s `[lints.rust]`, and `cargo doc --no-deps` must
   emit no warnings.
2. Prefer `#[expect(dead_code)]` over `#[allow(dead_code)]` for deliberate
   suppressions — `expect` makes the compiler flag the attribute itself once
   it's no longer needed, so stale suppressions can't silently accumulate.
3. All code must pass `cargo fmt --check`, `cargo clippy --all-targets -- -D
   warnings`, `cargo test`, and `cargo test --doc` with zero failures or
   warnings before it is committed.
4. Live tests (`tests/live.rs`) are double-gated: `#[ignore]` at the test
   level *and* a runtime check on `ENSEMBL_LIVE_TESTS=1`. Never remove either
   gate — the default `cargo test` run must stay fully offline.
5. Any doctest added to crate docs (`src/lib.rs`) or a public item must
   actually compile under `cargo test --doc`. Use ```` ```no_run ```` for
   anything that would hit the network; never use ```` ```ignore ````.

---

## 7. Quick reference: verifying a change

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --doc
cargo test --test parity   # only strictly required after touching ENDPOINTS/methods
cargo doc --no-deps        # confirm zero warnings before publishing docs changes
```
