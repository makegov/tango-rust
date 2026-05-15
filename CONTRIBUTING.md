# Contributing to tango-rust

Thanks for taking the time to contribute! `tango-rust` is the official Rust SDK for the [Tango API](https://makegov.com). It's deliberately small, idiomatic, and aligned with its sibling SDKs ([`tango-go`](https://github.com/makegov/tango-go), [`tango-node`](https://github.com/makegov/tango-node), and [`tango-python`](https://github.com/makegov/tango-python)).

## Ground rules

- The public API is stable. Breaking changes ship in major releases, not patches.
- New endpoints should land alongside their typed input/output structs and at least one example.
- Rustdoc on every public item (types, traits, functions, modules).
- No new external runtime dependencies without a discussion in an issue first.

## Local development

You'll need Rust 1.80+ (matches `rust-version` in the workspace `Cargo.toml`) and [`just`](https://github.com/casey/just) on your PATH. `rustfmt` and `clippy` come bundled with the toolchain via `rust-toolchain.toml`.

```bash
git clone https://github.com/makegov/tango-rust.git
cd tango-rust
just test     # unit tests
just lint     # cargo clippy --workspace --all-features -- -D warnings
just ci       # fmt-check + lint + test (matches CI)
```

Run `just` with no arguments to see the full list of recipes.

### Coverage

Coverage runs via [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov). Install it once with `cargo install cargo-llvm-cov`, then:

```bash
just cover       # generates lcov.info
just cover-html  # generates target/llvm-cov/html/index.html
```

### Integration tests

Integration tests are marked `#[ignore]` so they don't fire in normal `cargo test`. They live in the per-crate `tests/` directories and hit the live Tango API. They require `TANGO_API_KEY` in your environment.

```bash
TANGO_API_KEY=... just integration
```

## Branch and commit conventions

- Branch from `main`. Branch names: `<type>/<short-description>` (e.g. `feat/idv-subresources`, `fix/retry-jitter`).
- Use [Conventional Commits](https://www.conventionalcommits.org/) for commit messages. Common types: `feat`, `fix`, `chore`, `docs`, `refactor`, `test`.
- Reference the relevant issue in the body (e.g. `Closes #42`).

## Pull request checklist

- [ ] `just ci` is green locally.
- [ ] New public items have rustdoc comments.
- [ ] `CHANGELOG.md` has an entry under `## [Unreleased]` in the appropriate section (`Added`, `Changed`, `Fixed`, `Removed`).
- [ ] If the change is user-facing, `docs/` and `README.md` are updated.
- [ ] No new external runtime dependencies (or you've gotten a thumbs-up in an issue first).

## Releasing

Releases are tag-driven. Pushing a `vX.Y.Z` tag triggers `.github/workflows/release.yml`, which runs the test suite, publishes both workspace crates to crates.io (`tango-webhooks` first, then `tango`), and creates a GitHub Release with auto-generated notes. Use `just release-check` to dry-run the publish flow before tagging.

## Reporting issues

Bug? Feature request? Use the templates in `.github/ISSUE_TEMPLATE/`. For security issues, see [SECURITY.md](SECURITY.md) — do **not** open a public issue.
