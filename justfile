# Project Genesis - Justfile
# Single command runner for all build/test/lint operations

# Default recipe - show available commands
default:
    @just --list

# ============================================
# Build Commands
# ============================================

# Build all crates in debug mode
build:
    cargo build --workspace

# Build all crates in release mode
build-release:
    cargo build --workspace --release

# Build the main engine binary
build-engine:
    cargo build --package genesis-engine

# ============================================
# Test Commands
# ============================================

# Run all tests
test:
    cargo test --workspace

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run tests for a specific crate
test-crate CRATE:
    cargo test --package {{CRATE}}

# Run tests and generate coverage (requires cargo-llvm-cov)
coverage:
    cargo llvm-cov --workspace --html

# ============================================
# Lint Commands
# ============================================

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Apply formatting
fmt:
    cargo fmt --all

# Run clippy with warnings as errors
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Run all lints
lint: fmt-check clippy

# ============================================
# Validation (Agent Loop)
# ============================================

# Full validation - MUST PASS before commit
validate: fmt-check clippy test
    @echo "✅ All validations passed!"

# Auto-fix and validate
fix-and-validate:
    cargo fmt --all
    cargo clippy --workspace --all-targets --fix --allow-dirty
    cargo test --workspace
    @echo "✅ Fixed and validated!"

# ============================================
# Run Commands
# ============================================

# Run the engine in debug mode
run:
    cargo run --package genesis-engine

# Run the engine in release mode
run-release:
    cargo run --package genesis-engine --release

# Run with tracing enabled
run-trace:
    RUST_LOG=debug cargo run --package genesis-engine

# ============================================
# Documentation
# ============================================

# Generate documentation
doc:
    cargo doc --workspace --no-deps

# Generate and open documentation
doc-open:
    cargo doc --workspace --no-deps --open

# ============================================
# Cleaning
# ============================================

# Clean build artifacts
clean:
    cargo clean

# Clean and rebuild
rebuild: clean build

# ============================================
# Development Utilities
# ============================================

# Watch for changes and run tests
watch-test:
    cargo watch -x "test --workspace"

# Watch for changes and check
watch-check:
    cargo watch -x "check --workspace"

# Update dependencies
update:
    cargo update

# Check for outdated dependencies
outdated:
    cargo outdated

# Security audit
audit:
    cargo audit

# ============================================
# Benchmarks
# ============================================

# Run benchmarks
bench:
    cargo bench --workspace

# ============================================
# Release
# ============================================

# Create a release build with all optimizations
release:
    cargo build --workspace --release
    @echo "Release build complete: target/release/genesis"

# ============================================
# Git Helpers
# ============================================

# Pre-commit hook (validate before committing)
pre-commit: validate
    @echo "Ready to commit!"

# Show git status
status:
    git status

# ============================================
# Agent-specific commands
# ============================================

# Kernel agent validation
kernel-validate:
    cargo test --package genesis-kernel
    cargo clippy --package genesis-kernel -- -D warnings

# Gameplay agent validation
gameplay-validate:
    cargo test --package genesis-gameplay
    cargo clippy --package genesis-gameplay -- -D warnings

# Tools agent validation
tools-validate:
    cargo test --package genesis-tools
    cargo clippy --package genesis-tools -- -D warnings
