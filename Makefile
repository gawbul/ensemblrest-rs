.PHONY: all build test test-live test-coverage lint format format-check example clean

all: lint test build

build:
	cargo build --all-targets

test:
	cargo test

# There is deliberately no test-race target. Go's -race detector has no Rust
# equivalent because Send and Sync are checked at compile time; `make test`
# already covers what -race would catch. Do not add one.
test-live:
	ENSEMBL_LIVE_TESTS=1 cargo test --test live -- --ignored

# Requires cargo-llvm-cov: cargo install cargo-llvm-cov
test-coverage:
	cargo llvm-cov --html --open

lint:
	cargo clippy --all-targets -- -D warnings

format:
	cargo fmt

format-check:
	cargo fmt --check

example:
	cargo run --example basic

clean:
	cargo clean
