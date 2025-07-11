# Makefile for tsrc development and release builds

.PHONY: help build test clean release dev-build fmt clippy audit doc install cross-build

# Default target
help:
	@echo "Available targets:"
	@echo "  build       - Build in debug mode"
	@echo "  test        - Run tests"
	@echo "  clean       - Clean build artifacts"
	@echo "  release     - Build in release mode"
	@echo "  dev-build   - Run development build pipeline"
	@echo "  fmt         - Format code"
	@echo "  clippy      - Run clippy lints"
	@echo "  audit       - Run security audit"
	@echo "  doc         - Build documentation"
	@echo "  install     - Install locally"
	@echo "  cross-build - Build for all targets"
	@echo "  all         - Run full validation pipeline"
	@echo ""
	@echo "Documentation targets:"
	@echo "  docs        - Generate complete documentation"
	@echo "  docs-open   - Generate and serve documentation locally"
	@echo "  docs-clean  - Clean documentation artifacts"
	@echo "  docs-dev    - Build and open docs for development"
	@echo "  docs-check  - Check documentation for warnings"

# Development targets
build:
	cargo build

test:
	cargo test --all-features

clean:
	cargo clean

release:
	cargo build --release

dev-build:
	./scripts/dev-build.sh all

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

audit:
	cargo audit

doc:
	cargo doc --all-features --open

install:
	cargo install --path . --force

# Release targets
cross-build:
	./scripts/build-releases.sh

# CI-like validation
all: fmt clippy test doc release
	@echo "All validation steps completed successfully!"

# Quick development cycle
dev: fmt clippy test
	@echo "Development validation completed!"

# Performance testing
bench:
	cargo bench

# Generate code coverage
coverage:
	cargo tarpaulin --out html --output-dir target/coverage

# Check dependencies for updates
outdated:
	cargo outdated

# Update dependencies
update:
	cargo update

# Build examples
examples:
	cargo build --examples

# Run a specific example
example-%:
	cargo run --example $*

# Documentation targets
docs:
	./scripts/generate-docs.sh

docs-open: docs
	python3 -m http.server -d docs 8000

docs-clean:
	rm -rf docs target/doc

# Documentation development
docs-dev:
	cargo doc --open --all-features

docs-check:
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features