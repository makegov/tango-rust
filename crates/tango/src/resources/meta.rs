//! Meta endpoints — `/api/version/` and `/api/api-keys/`.
//!
//! These are top-level utility endpoints that don't fit under any one
//! resource family.

use crate::client::Client;
use crate::error::Result;
use crate::internal::apply_pagination;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;

/// Options for [`Client::list_api_keys`].
///
/// `/api/api-keys/` doesn't paginate in practice today, but we accept the
/// standard pagination/shape knobs anyway so callers can stay consistent
/// with every other list endpoint, and so the SDK doesn't need a breaking
/// change if the server starts paginating later.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListApiKeysOptions {
    /// 1-based page number (forwarded if the server starts paginating).
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size.
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Escape hatch for keys not first-classed here.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListApiKeysOptions {
    fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        apply_pagination(
            &mut q,
            self.page,
            self.limit,
            self.cursor.as_deref(),
            self.shape.as_deref(),
            self.flat,
            self.flat_lists,
        );
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

impl Client {
    /// `GET /api/version/` — return the API server's version metadata
    /// (build commit, deployed-at, etc.).
    ///
    /// Single-value endpoint: the server returns a JSON object, not a
    /// paginated envelope, so this returns a [`Record`] directly.
    pub async fn get_version(&self) -> Result<Record> {
        self.get_json::<Record>("/api/version/", &[]).await
    }

    /// `GET /api/api-keys/` — return the authenticated caller's API keys
    /// and associated metadata.
    ///
    /// The server returns a single structured object today (no `count` /
    /// `results` envelope), so this returns a [`Record`] rather than a
    /// [`Page`](crate::Page). Mirrors the Go SDK's `ListAPIKeys` shape.
    pub async fn list_api_keys(&self, opts: ListApiKeysOptions) -> Result<Record> {
        let q = opts.to_query();
        self.get_json::<Record>("/api/api-keys/", &q).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_q(q: &[(String, String)], k: &str) -> Option<String> {
        q.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.clone())
    }

    #[test]
    fn api_keys_opts_emit_pagination() {
        let opts = ListApiKeysOptions::builder()
            .page(3u32)
            .limit(25u32)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "page").as_deref(), Some("3"));
        assert_eq!(get_q(&q, "limit").as_deref(), Some("25"));
    }

    #[test]
    fn api_keys_opts_empty_default() {
        let opts = ListApiKeysOptions::default();
        assert!(opts.to_query().is_empty());
    }

    #[test]
    fn api_keys_opts_extra_passthrough() {
        let mut extra = BTreeMap::new();
        extra.insert("scope".into(), "read".into());
        let opts = ListApiKeysOptions::builder().extra(extra).build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "scope").as_deref(), Some("read"));
    }
}
