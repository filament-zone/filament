.PHONY: help

help: ## Display this help message
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

test:  ## Runs test suite using next test
	@cargo nextest run --workspace --all-features --status-level skip

test-default-features:  ## Runs test suite using default features
	@cargo nextest run --workspace --status-level skip

install-dev-tools:  ## Installs all necessary cargo helpers
	cargo install cargo-llvm-cov
	cargo install cargo-hack
	cargo install cargo-udeps
	cargo install flaky-finder
	cargo install cargo-nextest --locked
	cargo install cargo-risczero
	cargo risczero install
	cargo install zepter
	rustup target add thumbv6m-none-eabi
	rustup target add wasm32-unknown-unknown


install-risc0-toolchain:
	cargo risczero install --version v2024-04-22.0
	@echo "Risc0 toolchain version:"
	cargo +risc0 --version
