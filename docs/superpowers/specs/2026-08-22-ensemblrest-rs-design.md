# ensemblrest-rs — Design

**Date:** 2026-08-22
**Status:** Approved for planning

A Rust client library for the [Ensembl REST API](https://rest.ensembl.org/), ported from
[`goensemblrest`](https://github.com/gawbul/goensemblrest), itself a port of
[`pyEnsemblRest`](https://github.com/gawbul/pyEnsemblRest).

The goal is endpoint parity and behavioural parity with the Go port, following its
patterns wherever they carry over, and diverging only where Rust has a materially
better answer. Each divergence is called out explicitly in this document.

---

## 1. Constraints and the dependency decision

The instruction was to prefer the standard library and take external crates only where
absolutely necessary. This constrains Rust far more tightly than it constrains Go.

`goensemblrest` has zero dependencies because Go's standard library ships `net/http`
(with TLS) and `encoding/json`. Rust's standard library ships neither, and provides no
TLS at all — `std::net::TcpStream` is the ceiling. Since `rest.ensembl.org` is
HTTPS-only, a literally zero-dependency Rust port would require hand-writing an HTTP/1.1
client *and* a TLS implementation, which is not defensible.

**Decision — three direct dependencies, no more:**

| Crate | Version | Why it cannot come from std |
|---|---|---|
| `ureq` | 3.4 | Blocking HTTP/1.1 client; bundles `rustls` for TLS. No async runtime imposed on consumers. |
| `serde` | 1.0 | Derive-based deserialization. Rust has no runtime reflection, so Go's `target any` has no std equivalent. |
| `serde_json` | 1.0 | JSON parsing/serialization. |

Everything else is hand-written on `std`, specifically including:

- `{{param}}` template scanning — replaces Go's `regexp`, no `regex` crate.
- Percent-encoding for path segments and query strings — no `percent-encoding` or `url` crate.
- The sliding-window rate limiter — `Mutex<VecDeque<Instant>>`.
- Retry backoff — `std::thread::sleep`.
- The test HTTP server — `std::net::TcpListener`, so there are **zero dev-dependencies**.

**Toolchain:** Rust 1.98.0 (latest stable, released 2026-08-18), edition 2024,
`rust-version = "1.98"` in `Cargo.toml`.

**Crate name:** `ensemblrest`, in repository `ensemblrest-rs`.

---

## 2. Naming conventions

- Methods are snake_case of the Go names: `GetLookupByID` → `get_lookup_by_id`.
- **Endpoint table keys stay camelCase** (`"getLookupById"`), matching `goensemblrest`
  and `pyEnsemblRest`. These strings are the cross-port dynamic-dispatch contract and
  must not be re-cased.
- Struct fields are snake_case with `#[serde(rename)]` where the wire name differs
  (e.g. `maf` ← `"MAF"`).

---

## 3. Crate layout

Mirrors `goensemblrest` file-for-file so the two ports stay diffable side by side.

```
ensemblrest-rs/
├── Cargo.toml                edition 2024, rust-version 1.98, 3 deps
├── Makefile
├── README.md                 quickstart + full endpoint catalog
├── LICENSE                   MIT, Copyright (c) 2020-2026 Steve Moss
├── AGENTS.md / CLAUDE.md
├── .github/workflows/{pull_request,nightly,push_tag}.yaml
├── examples/basic.rs
├── src/
│   ├── lib.rs                crate docs, re-exports, VERSION, DEFAULT_* consts
│   ├── client.rs             Client, ClientBuilder
│   ├── request.rs            path resolution, encoding, retry loop
│   ├── ratelimit.rs          sliding-window limiter, header telemetry
│   ├── error.rs              Error, ApiError, ApiErrorKind, status table
│   ├── response.rs           Response
│   ├── options.rs            RequestOption
│   ├── endpoints.rs          EndpointSpec, ENDPOINTS, index, call(), endpoints()
│   ├── types.rs              serde domain models
│   └── <16 domain modules>   archive, comparative, ga4gh, info, ld, lookup,
│                             mapping, ontology, overlap, phenotype, regulation,
│                             sequence, transcript, variation, vep, xrefs
└── tests/
    ├── common/mock.rs        std-only HTTP/1.1 mock server
    ├── client.rs
    ├── endpoints.rs
    └── live.rs
```

### Domain module sizes (methods per module, from the Go port)

| Module | Methods | Module | Methods |
|---|---:|---|---:|
| archive | 2 | overlap | 3 |
| comparative | 9 | phenotype | 4 |
| ga4gh | 22 | regulation | 1 |
| info | 26 | sequence | 4 |
| ld | 3 | transcript | 1 |
| lookup | 4 | variation | 6 |
| mapping | 4 | vep | 6 |
| ontology | 8 | xrefs | 3 |

**Total: 106 endpoints, 106 typed methods.** Verified against the Go source: the
endpoint table and the method set are in exact 1:1 correspondence, with no table entry
lacking a method and no method referencing a missing entry. The Rust port must preserve
this invariant, enforced by a test (§10).

---

## 4. Client and configuration

```rust
pub struct Client { inner: Arc<Inner> }   // Clone + Send + Sync

struct Inner {
    agent:        ureq::Agent,
    base_url:     String,          // trailing slashes trimmed
    user_agent:   String,
    headers:      Vec<(String, String)>,
    max_attempts: u32,
    wall_time:    Duration,
    max_response_bytes: u64,
    limiter:      RateLimiter,
}
```

Built through `ClientBuilder`, replacing Go's functional options:

```rust
let client = Client::builder()
    .base_url("https://rest.ensembl.org")
    .timeout(Duration::from_secs(60))
    .rate_limit(15, Duration::from_secs(1))
    .max_attempts(5)
    .user_agent("custom-agent/1.0")
    .header("X-Custom", "value")
    .max_response_bytes(100 * 1024 * 1024)
    .agent(custom_ureq_agent)
    .build()?;
```

`Client::new()` is shorthand for `Client::builder().build()`.

**Divergence — no `RwLock` on `Client`.** Go guards `Client` with `sync.RWMutex`
because its options mutate the struct after construction. The builder makes
configuration immutable once built, so the only remaining lock is the rate limiter's
own mutex. `Client: Clone` is an `Arc` bump, so clones share one limiter and one
connection pool — the same semantics as passing a shared `*Client` around goroutines.

### Defaults (identical to the Go port)

| Constant | Value |
|---|---|
| `DEFAULT_BASE_URL` | `https://rest.ensembl.org` |
| `DEFAULT_CONTENT_TYPE` | `application/json` |
| `DEFAULT_TIMEOUT` | 60 s |
| `DEFAULT_MAX_ATTEMPTS` | 5 |
| `DEFAULT_REQS_PER_SEC` | 15 |
| `DEFAULT_WALL_TIME` | 1 s |
| `DEFAULT_MAX_RESPONSE_BYTES` | 100 MiB |
| `DEFAULT_USER_AGENT` | `ensemblrest/{VERSION} (Rust 1.98; +https://github.com/gawbul/ensemblrest-rs)` |

Builder validation rejects non-positive timeout, `reqs_per_sec`, window, and
`max_attempts < 1`, returning `Error::InvalidConfig`.

**Addition beyond the Go port — `max_response_bytes`.** `ureq` caps response bodies at
10 MiB by default and returns an error above it. Several Ensembl endpoints exceed that
routinely — large `overlap` result sets, multi-region `sequence` POSTs, and
`alignment/region` blocks. The default is therefore raised to 100 MiB via
`body_mut().with_config().limit(..)` on every read, and exposed as a builder option so
callers can raise or lower it. Go has no equivalent because `net/http` imposes no such
cap; without this the port would fail on exactly the large queries users care about.

**Divergence — no cancellation context.** Go threads `ctx context.Context` through
every method, using it to abort mid-sleep in the limiter and in retry backoff. Blocking
Rust has no equivalent; timeouts are configured on the client and enforced by `ureq`.
Accepted trade-off: a caller cannot abort a request that is sitting in a rate-limiter or
backoff sleep — worst case roughly 20 s at default settings. Should this become a real
problem, an opt-in `Cancel` handle (`Arc<AtomicBool>`) checked at sleep boundaries can
be added later as a `RequestOption` without breaking any signature.

---

## 5. Request engine (`request.rs`)

### Path resolution

`resolve_path(template, params) -> Result<String>` scans for `{{name}}` with a
hand-written parser (no `regex` crate) and substitutes percent-encoded values.

**Critical invariant:** colons must survive unescaped. Genomic coordinates such as
`13:32889611..32973805:1` and species-qualified symbols such as `homo_sapiens:BRCA2`
break if `:` becomes `%3A`. The encoder therefore passes through the RFC 3986 unreserved
set (`ALPHA`, `DIGIT`, `-`, `.`, `_`, `~`) plus `:`, and escapes everything else. `.`
being unreserved means the `..` range syntax survives naturally. This mirrors Go's
`url.PathEscape` followed by its explicit `%3A` → `:` fix-up.

A missing or empty parameter returns `Error::MissingParam(name)` before any network
call, matching Go's `mandatory param %q not specified`.

### Query strings

Hand-written to match `url.Values.Encode()` exactly: keys sorted alphabetically,
`application/x-www-form-urlencoded` escaping, space as `+`. Byte-identical URLs across
the Rust and Go ports keeps mock-server assertions portable between the two suites.

### Retry loop

Reproduces `executeRequest` step for step:

1. `limiter.wait()` — enforce the sliding window.
2. Build the request; apply client headers, then per-request headers, then
   `User-Agent`, then `Content-Type` and `Accept`, both set to the effective content
   type. **Effective content type** resolves in this order: the per-request
   `content_type()` option if given, else the endpoint spec's `content_type`, else
   `DEFAULT_CONTENT_TYPE`. `User-Agent` is applied after per-request headers and so
   cannot be overridden per call, matching Go.
3. Send. On 2xx, return `Response`.
4. Otherwise classify and either return or retry.

**Transient (retried):**
- HTTP 408, 500, 502, 503, 504 — always.
- HTTP 429 — always, honouring `Retry-After`.
- HTTP 400 — only when the body contains one of these, case-insensitively:
  - `something bad has happened`
  - `Something went wrong while fetching from LDFeatureContainerAdaptor`
  - `timeout`
- Transport/network errors — treated as transient.

**Backoff:** `Retry-After` seconds when present, otherwise
`attempt × wall_time × 2` with a 10 ms floor.

**Exhaustion:** `Error::MaxRetries { attempts, last }`, where `last` boxes the final
underlying error.

---

## 6. Rate limiting (`ratelimit.rs`)

Sliding window over `Mutex<VecDeque<Instant>>`, default 15 requests per second.
`wait()` prunes entries older than the window, admits immediately if under capacity,
otherwise sleeps until the oldest entry expires and re-checks.

`update_from_headers` parses telemetry after every response:
`X-RateLimit-Reset`, `X-RateLimit-Limit`, `X-RateLimit-Remaining`,
`X-RateLimit-Period`, `Retry-After`. Exposed through `client.rate_limit()` and on
`Response`/`ApiError`.

```rust
pub struct RateLimitInfo {
    pub reset:       Option<i64>,
    pub limit:       Option<i64>,
    pub remaining:   Option<i64>,
    pub period:      Option<i64>,
    pub retry_after: Option<f64>,
}
```

---

## 7. Errors (`error.rs`)

```rust
pub enum Error {
    Api(ApiError),
    MaxRetries { attempts: u32, last: Box<Error> },
    MissingParam(String),
    UnknownEndpoint(String),
    Transport(ureq::Error),
    Decode(serde_json::Error),
    InvalidConfig(String),
}

pub struct ApiError {
    pub status:     u16,
    pub message:    String,
    pub rate_limit: RateLimitInfo,
    pub body:       Vec<u8>,
}

pub enum ApiErrorKind {
    BadRequest, NotFound, Timeout, RateLimit,
    InternalServer, ServiceUnavailable, Other(u16),
}
```

Go's sentinel-plus-`errors.Is` pattern maps onto `Error::api_kind() -> Option<ApiErrorKind>`
with `is_not_found()`-style conveniences. `impl std::error::Error::source()` chains
`MaxRetries → Api`, preserving Go's `Unwrap()` behaviour.

**`Display` output is byte-identical to the Go and Python ports:**

```
EnsEMBL REST API returned a 404 (Not Found): <message>
EnsEMBL REST API returned a 429 (Too Many Requests): <message> (Rate limit hit: Retry after 3 seconds)
```

`HTTP_STATUS_DESCRIPTIONS` ports across unchanged (200, 400, 403, 404, 408, 415, 429,
500, 503). Error message extraction tries JSON `error`, then JSON `message`, then the
raw body, then the status text — matching `parseErrorMessage`.

`pub type Result<T> = std::result::Result<T, Error>;`

---

## 8. Response, options, endpoints

### Response (`response.rs`)

```rust
pub struct Response { /* status, content_type, rate_limit, body */ }

impl Response {
    pub fn status(&self) -> u16;
    pub fn content_type(&self) -> Option<&str>;
    pub fn rate_limit(&self) -> &RateLimitInfo;
    pub fn bytes(&self) -> &[u8];
    pub fn into_bytes(self) -> Vec<u8>;
    pub fn text(&self) -> Result<String>;                       // UTF-8 checked
    pub fn json<T: DeserializeOwned>(&self) -> Result<T>;
}
```

**Divergence — `Response` handle instead of Go's `target any` out-parameter.** Rust
has no runtime reflection, so `target any` cannot be transliterated. Returning a
`Response` gives one uniform path for JSON and for the non-JSON content types Ensembl
serves (`text/x-fasta`, `text/x-gff3`, `text/x-phyloxml`, `text/x-nh`), which
`goensemblrest` handles by special-casing `*string` and `*[]byte` inside
`unmarshalResponse`. It also exposes status and rate-limit headers, and matches the
shape Rust users already know from `reqwest` and `ureq`. Cost: two `?` per call.

```rust
let rec: LookupRecord = client.get_lookup_by_id("ENSG00000157764", &[])?.json()?;
let fasta: String = client
    .get_sequence_by_id("ENSG00000157764", &[content_type("text/x-fasta")])?
    .text()?;
let v: serde_json::Value = client.get_info_species(&[])?.json()?;
```

### Request options (`options.rs`)

```rust
pub enum RequestOption<'a> {
    Query(&'a str, &'a str),
    ContentType(&'a str),
    Header(&'a str, &'a str),
}
pub fn query(k: &str, v: &str) -> RequestOption<'_>;
pub fn content_type(ct: &str) -> RequestOption<'_>;
pub fn header(k: &str, v: &str) -> RequestOption<'_>;
```

Passed as `&[RequestOption]`; the common `&[]` case allocates nothing.

**Divergence — three constructors instead of Go's five.** `WithQuery`,
`WithQueryParams` and `WithURLValues` all collapse into repeated `query()`, which
appends exactly as `url.Values.Add` does. No expressiveness is lost.

### Endpoint table (`endpoints.rs`)

```rust
pub struct EndpointSpec {
    pub name:            &'static str,
    pub doc:             &'static str,
    pub url:             &'static str,
    pub method:          Method,
    pub content_type:    &'static str,
    pub post_parameters: &'static [&'static str],
}

pub enum Method { Get, Post }

pub static ENDPOINTS: &[EndpointSpec] = &[ /* 106 entries */ ];
pub fn endpoints() -> &'static HashMap<&'static str, &'static EndpointSpec>;  // OnceLock
```

**Divergence — `endpoints()` returns a static reference, not a clone.** Go clones the
map defensively; the Rust data is immutable `'static`, so there is nothing to defend
against.

### Dispatch

Typed methods, one per endpoint, are thin delegates:

```rust
impl Client {
    /// Finds the species and database for a single identifier.
    pub fn get_lookup_by_id(&self, id: &str, opts: &[RequestOption]) -> Result<Response> {
        self.call("getLookupById", &[("id", id)], None, opts)
    }

    /// Finds the species and database for several identifiers.
    pub fn get_lookup_by_multiple_ids(&self, ids: &[&str], opts: &[RequestOption]) -> Result<Response> {
        self.call("getLookupByMultipleIds", &[], Some(&json!({ "ids": ids })), opts)
    }
}
```

Dynamic dispatch, mirroring Go's `Call` and pyEnsemblRest's attribute dispatch:

```rust
pub fn call(
    &self,
    name: &str,
    path_params: &[(&str, &str)],
    body: Option<&serde_json::Value>,
    opts: &[RequestOption],
) -> Result<Response>;
```

Unknown names yield `Error::UnknownEndpoint`.

**POST body keys are per-endpoint and must be copied exactly.** `getArchiveByMultipleIds`
uses `"id"` while `getLookupByMultipleIds` uses `"ids"`. This asymmetry is real Ensembl
API behaviour and must be preserved, not tidied.

---

## 9. Domain types (`types.rs`)

All models from `types.go` port over with
`#[derive(Debug, Clone, Default, Serialize, Deserialize)]`:
`PingResponse`, `ArchiveRecord`, `LookupRecord`, `SequenceRecord`, `SpeciesRecord`,
`SpeciesResponse`, `AssemblyInfo`, `AssemblyRegionInfo`, `HomologyRecord`,
`HomologyResponse`, `XrefRecord`, `VariationRecord`, `LDRecord`, `MappingRecord`,
`VEPRecord`, `BeaconResponse`, `BeaconQueryResponse`.

Mapping rules:

| Go | Rust |
|---|---|
| `*float64`, `*int`, `*bool` | `Option<f64>`, `Option<i64>`, `Option<bool>` |
| `json.RawMessage` | `serde_json::Value` |
| `[]string` | `Vec<String>` |
| `omitempty` scalar | plain field under container-level `#[serde(default)]` |

`#[serde(default)]` on every container is mandatory, not cosmetic: Ensembl's response
shape varies with query parameters, and a missing field must deserialize to a default
rather than fail the whole call.

**Deliberate fix, not a faithful copy.** `LookupRecord.Extra json.RawMessage` is tagged
`json:"-"` in Go, meaning it can never be populated — dead code. Rust uses
`#[serde(flatten)] pub extra: Map<String, Value>`, which delivers what the field was
reaching for: the additional nested data returned under `?expand=1` becomes reachable
instead of silently dropped.

---

## 10. Testing

**Zero dev-dependencies.** `tests/common/mock.rs` is a `std::net::TcpListener` bound to
`127.0.0.1:0` with a worker thread: it parses the request line, headers and
`Content-Length` body, replies from a scripted queue of `(status, headers, body)`, and
records received requests for assertion. Plain `http://`, so no TLS is involved. This is
the direct analogue of Go's `httptest.Server`, in roughly 150 lines.

| Suite | Coverage |
|---|---|
| `tests/endpoints.rs` | All 106 typed methods against the mock, asserting HTTP method, resolved path, query string and request body. |
| `tests/endpoints.rs` (parity) | Every `ENDPOINTS` key is exercised by exactly one typed method, and every method targets a key that exists. Enforces the 106↔106 invariant from §3. |
| `tests/client.rs` | Builder validation; limiter timing (N+1 requests take at least one window); the full retry matrix — transient 400 vs fatal 400, 429 with `Retry-After`, 503, transport error, exhaustion; colon preservation in paths; query encoding order; `Display`/`api_kind`/`source` on errors; every `Response` decode path including invalid UTF-8 and malformed JSON. |
| `tests/live.rs` | Smoke tests against `rest.ensembl.org`, `#[ignore]` and gated on `ENSEMBL_LIVE_TESTS=1`. |
| Rustdoc doctests | Replace Go's `example_test.go`. Network-touching examples marked `no_run`. |

---

## 11. Repository furniture

### Makefile

| Target | Command |
|---|---|
| `all` | `lint test build` |
| `build` | `cargo build --all-targets` |
| `test` | `cargo test` |
| `test-live` | `ENSEMBL_LIVE_TESTS=1 cargo test --test live -- --ignored` |
| `test-coverage` | `cargo llvm-cov --html` |
| `lint` | `cargo clippy --all-targets -- -D warnings` |
| `format` | `cargo fmt` |
| `example` | `cargo run --example basic` |
| `clean` | `cargo clean` |

**Divergence — no `test-race` target.** Go's `-race` detector has no Rust equivalent
because `Send`/`Sync` are compile-time guarantees. `make test` absorbs it.
`cargo llvm-cov` is a developer tool, not a crate dependency.

### CI workflows

- **`pull_request.yaml`** — `cargo fmt --check`, `cargo clippy -- -D warnings`,
  `cargo test`, `cargo build --all-targets`, plus live smoke tests with
  `continue-on-error: true` so third-party outages don't block PRs.
- **`nightly.yaml`** — 03:00 UTC live API drift check against `rest.ensembl.org`.
- **`push_tag.yaml`** — on `v*` tags: full test suite, `cargo publish` to crates.io
  using a `CARGO_REGISTRY_TOKEN` secret, and a GitHub Release with generated notes.

### Docs

`README.md` with quickstart, configuration, error handling and the full 106-endpoint
catalog; `examples/basic.rs`; MIT `LICENSE` (Copyright (c) 2020-2026 Steve Moss);
`AGENTS.md` and `CLAUDE.md` adapted from the Go port.

---

## 12. Summary of divergences from `goensemblrest`

| # | Go | Rust | Reason |
|---|---|---|---|
| 1 | Zero dependencies | 3 direct crates | Rust std has no HTTP, TLS or JSON |
| 2 | `target any` out-param | `Response` + `.json()`/`.text()`/`.bytes()` | No runtime reflection; unifies JSON and FASTA/GFF3 |
| 3 | `ctx context.Context` per method | Client-level timeouts | No blocking-Rust equivalent; opt-in `Cancel` deferred |
| 4 | Functional options | `ClientBuilder` | Idiomatic Rust; makes config immutable |
| 5 | `sync.RWMutex` on `Client` | `Arc` + immutable config | Builder removes post-construction mutation |
| 6 | 5 request-option constructors | 3 | `query()` repeated covers all three query variants |
| 7 | `Endpoints()` clones the map | Returns `&'static` | Data is immutable; nothing to defend |
| 8 | `regexp` for `{{param}}` | Hand-written scanner | Avoids the `regex` crate |
| 9 | Sentinel errors + `errors.Is` | `Error` enum + `api_kind()` | Idiomatic Rust; `source()` preserves unwrapping |
| 10 | `LookupRecord.Extra` is dead (`json:"-"`) | `#[serde(flatten)] extra` | Fixes a latent bug rather than copying it |
| 11 | `make test-race` | Folded into `make test` | `Send`/`Sync` are compile-time |
| 12 | No publish step | `cargo publish` on tag | crates.io has no module proxy |

---

## 13. Out of scope

- Async support. The design keeps a `Transport` seam internally, so a `reqwest`/`tokio`
  backend could be feature-gated later without disturbing the endpoint table, types,
  errors or rate limiter.
- Response-schema modelling beyond what `types.go` already covers. `.json::<T>()`
  accepts any `DeserializeOwned`, including `serde_json::Value`, so unmodelled endpoints
  remain fully usable.
- Changes to `goensemblrest` or `pyEnsemblRest`.
