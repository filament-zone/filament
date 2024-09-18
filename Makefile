.PHONY: help

help: ## Display this help message
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

test:  ## Runs test suite using next test
	@cargo nextest run --workspace --no-default-features --features mock_da --features native --all-targets --status-level skip

install-dev-tools:  ## Installs all necessary cargo helpers
install-dev-tools: install-risc0-toolchain
	rustup update nightly
	cargo install cargo-llvm-cov
	cargo install cargo-hack
	cargo install --locked cargo-udeps
	cargo install cargo-deny
	cargo install flaky-finder
	cargo install cargo-nextest --locked
	cargo install zepter
	cargo install wasm-pack
	rustup target add wasm32-unknown-unknown

install-risc0-toolchain:
	cargo install cargo-risczero
	cargo risczero install --version r0.1.79.0
	@echo "Risc0 toolchain version:"
	cargo +risc0 --version

install-sp1-toolchain: ## FIXME(xla): Currently fails with segfault when invoking sp1up.
	curl -L https://raw.githubusercontent.com/succinctlabs/sp1/main/sp1up/install | bash
	~/.sp1/bin/sp1up --token "$$GITHUB_TOKEN"
	~/.sp1/bin/cargo-prove prove --version
	~/.sp1/bin/cargo-prove prove install-toolchain
	@echo "SP1 toolchain version:"
	cargo +succinct --version

fmt:
	cargo +nightly fmt --all --check

lint:  ## cargo check and clippy. Skip clippy on guest code since it's not supported by risc0
	## fmt first, because it's the cheapest
	SKIP_GUEST_BUILD=1 cargo check
	SKIP_GUEST_BUILD=1 cargo check --features celestia_da --no-default-features
	SKIP_GUEST_BUILD=1 cargo clippy --workspace --no-deps -- -Dwarnings -Dunused -Dfuture-incompatible -Drefining-impl-trait -Dnonstandard-style -Drust-2018-idioms -Drust-2021-compatibility
	SKIP_GUEST_BUILD=1 cargo clippy --workspace --features celestia_da --no-default-features --no-deps -- -Dwarnings -Dunused -Dfuture-incompatible -Drefining-impl-trait -Dnonstandard-style -Drust-2018-idioms -Drust-2021-compatibility
	## Invokes Zepter multiple times because fixes sometimes unveal more underlying issues.
	zepter
	zepter
	zepter

lint-fix:  ## cargo fmt, fix and clippy. Skip clippy on guest code since it's not supported by risc0
	cargo +nightly fmt --all
	cargo fix --allow-dirty
	SKIP_GUEST_BUILD=1 cargo clippy --fix --allow-dirty

find-unused-deps: ## Prints unused dependencies for project. Note: requires nightly
	cargo +nightly udeps --all-targets --all-features

find-flaky-tests:  ## Runs tests over and over to find if there's flaky tests
	flaky-finder -j16 -r320 --continue "cargo test -- --nocapture"

build-wasm-dev:
	wasm-pack build --dev --no-opt --target web crates/wasm

build-wasm-release:
	wasm-pack build --release --target web crates/wasm

pack-wasm:
	wasm-pack pack crates/wasm

test-wasm-chrome:
	wasm-pack test --chrome --headless crates/wasm

test-wasm-firefox:
	wasm-pack test --firefox --headless crates/wasm
