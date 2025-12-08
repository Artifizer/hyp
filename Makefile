.PHONY: help build clippy kani test clean check all

.DEFAULT_GOAL := help

## Show this help message
help:
	@echo "Available targets:"
	@awk '/^##/ {desc=substr($$0,4); next} /^[a-zA-Z_-]+:/ && desc {printf "  \033[36m%-15s\033[0m %s\n", $$1, desc; desc=""}' $(MAKEFILE_LIST)

## Build all workspace crates
build:
	@echo "Building all workspace crates..."
	RUSTFLAGS="-D warnings" cargo build --workspace
	RUSTFLAGS="-D warnings" cargo build --workspace --release

## Run clippy linter on all crates
clippy:
	@echo "Running clippy..."
	@command -v cargo-clippy >/dev/null 2>&1 || { \
		echo "Clippy not found, installing..."; \
		rustup component add clippy; \
	}
	@echo "Checking hyp-checks-generic (strict)..."
	@cd crates/hyp-checks-generic && cargo clippy --all-targets -- -D warnings
	@echo "Checking hyp (strict)..."
	@cd crates/hyp && cargo clippy --bins -- -D warnings
	@echo ""
	@echo "OK. Clippy checks passed for analyzer and CLI crates"

## Run clippy on the hyp-examples crate that intentionally containst problematic code for demonstration
clippy-examples:
	@echo "Running clippy..."
	@command -v cargo-clippy >/dev/null 2>&1 || { \
		echo "Clippy not found, installing..."; \
		rustup component add clippy; \
	}
	@echo "Note: hyp-examples crate intentionally contains problematic code for demonstration"
	@# Enabling restriction group as requested. Note: this includes conflicting lints!
	@cd crates/hyp-examples && cargo clippy --all-targets --lib -- -W clippy::restriction -W clippy::pedantic -W clippy::nursery -W clippy::cargo -A clippy::blanket_clippy_restriction_lints || true
	@echo ""
	@echo "Checking hyp-examples-cli (allowing warnings from hyp-examples)..."
	@cd crates/hyp-examples-cli && cargo clippy --bins 2>&1 | grep -v "hyp-examples" | grep -E "^(warning|error):" || echo "  OK. No issues in CLI code"


## Run Kani formal verifier
kani:
	@command -v kani >/dev/null || \
		(echo "Installing Kani verifier..." && \
		 cargo install --locked kani-verifier && cargo kani setup)
	cargo kani --workspace --all-features --output-format terse

## Run all tests in the workspace
test:
	@echo "Running unit tests..."
	RUSTFLAGS="-D warnings" cargo test --workspace
	@echo "========================================================================"
	@echo "Ensuring problem exampels are executable w/o problems..."
	cargo run --bin hyp-examples -- run-all

## Run code coverage for analyzer and CLI crates
coverage:
	@echo "Running code coverage..."
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { \
		echo "cargo-llvm-cov not found, installing..."; \
		cargo install cargo-llvm-cov; \
	}
	@echo "Generating coverage report for hyp-checks-generic and hyp crates..."
	cargo llvm-cov --package hyp-checks-generic --package hyp --html --open

## Run code coverage and generate lcov format for CI/CD
coverage-lcov:
	@echo "Running code coverage (lcov format)..."
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { \
		echo "cargo-llvm-cov not found, installing..."; \
		cargo install cargo-llvm-cov; \
	}
	@echo "Generating coverage report in lcov format..."
	cargo llvm-cov --package hyp-checks-generic --package hyp --lcov --output-path coverage.lcov
	@echo "Coverage report saved to coverage.lcov"

## Run code coverage with verbose output
coverage-verbose:
	@echo "Running code coverage with verbose output..."
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { \
		echo "cargo-llvm-cov not found, installing..."; \
		cargo install cargo-llvm-cov; \
	}
	@echo "Generating detailed coverage report..."
	cargo llvm-cov --package hyp-checks-generic --package hyp --html
	@echo ""
	@echo "Coverage summary:"
	cargo llvm-cov --package hyp-checks-generic --package hyp report --summary-only

## Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Cleaning coverage artifacts..."
	cargo llvm-cov clean 2>/dev/null || true
	@rm -f coverage.lcov

## Run cargo check on all crates
check:
	@echo "Running cargo check..."
	cargo check --workspace

all: clean build clippy test check
