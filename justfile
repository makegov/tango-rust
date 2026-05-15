# justfile for github.com/makegov/tango-rust
#
# Common dev recipes. Mirrors the script ergonomics of tango-go's Makefile
# and tango-node's package.json.
#
# Run `just` (no args) to list available recipes.

# Default: list recipes
default:
    @just --list

# Run unit tests (default feature set, workspace)
test:
    cargo test --workspace

# Run all tests with every feature enabled
test-all:
    cargo test --workspace --all-features

# Format Rust source in-place
fmt:
    cargo fmt --all

# Check formatting (CI mode)
fmt-check:
    cargo fmt --all -- --check

# Run clippy with warnings denied (matches CI)
lint:
    cargo clippy --workspace --all-features -- -D warnings

# Generate lcov.info coverage report (used by Codecov upload)
cover:
    cargo llvm-cov --workspace --lcov --output-path lcov.info

# Generate HTML coverage report (open `target/llvm-cov/html/index.html`)
cover-html:
    cargo llvm-cov --workspace --html

# Build and open rustdoc for the workspace
doc:
    cargo doc --workspace --no-deps --open

# Run integration tests (live API). They live behind #[ignore] so they
# don't fire in normal `cargo test`. Requires TANGO_API_KEY.
integration:
    cargo test --workspace -- --ignored

# Everything CI runs locally
ci: fmt-check lint test

# Dry-run the publish flow for both crates (catches version/metadata issues
# before tagging). tango-webhooks goes first to match release.yml.
release-check:
    cargo publish --dry-run -p makegov-tango-webhooks
    cargo publish --dry-run -p makegov-tango

# Install pre-commit + pre-push git hooks. Requires uv (https://docs.astral.sh/uv/).
# Hooks mirror what CI runs: fmt + check on commit, clippy on push.
hooks:
    @command -v uv >/dev/null 2>&1 || (echo "uv not on PATH. Install with: brew install uv" && exit 1)
    uv tool install --upgrade pre-commit
    pre-commit install
    pre-commit install --hook-type pre-push
