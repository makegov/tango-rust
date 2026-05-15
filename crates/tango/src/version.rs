//! Version constant. Bumped in lockstep with `Cargo.toml` and the git tag.

/// The current SDK version. Kept in sync with `Cargo.toml` and the `CHANGELOG`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
