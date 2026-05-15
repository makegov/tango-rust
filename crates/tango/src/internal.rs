//! Internal helpers shared across resource modules.
//!
//! [`ListOptions`] is the shared pagination/shaping field set every list
//! endpoint accepts. Resource-specific option types compose it via the
//! `#[serde(flatten)] pagination: ListOptions` pattern, so callers don't have
//! to repeat `page` / `limit` / `cursor` per resource.

use bon::Builder;

/// Pagination, shape, and flattening options common to every list endpoint.
///
/// Resource-specific option builders compose this via a `pagination` field.
/// Constructing one directly is rarely useful; use the resource builder
/// (e.g. `ListContractsOptions::builder()`) and call `.page(...)`,
/// `.limit(...)`, `.cursor(...)`, `.shape(...)`, etc. on it.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListOptions {
    /// 1-based page number for offset-paginated endpoints. Mutually exclusive
    /// with [`cursor`](Self::cursor); when both are set, the cursor wins.
    #[builder(into)]
    pub page: Option<u32>,

    /// Page size. The server caps this at 100 on most endpoints.
    #[builder(into)]
    pub limit: Option<u32>,

    /// Keyset cursor for cursor-paginated endpoints. Pass the `cursor` field
    /// from the previous [`Page`](crate::Page).
    #[builder(into)]
    pub cursor: Option<String>,

    /// Comma-separated field selector for dynamic response shaping. Use one of
    /// the `SHAPE_*` constants or roll your own.
    #[builder(into)]
    pub shape: Option<String>,

    /// When `true`, collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,

    /// When `true` (and [`flat`](Self::flat) is also `true`), flatten
    /// list-valued nested fields as well.
    #[builder(default)]
    pub flat_lists: bool,
}

impl ListOptions {
    /// Apply this option set to a query-pair list. Resource methods call this
    /// then layer their resource-specific filters on top.
    pub(crate) fn apply(&self, q: &mut Vec<(String, String)>) {
        apply_pagination(
            q,
            self.page,
            self.limit,
            self.cursor.as_deref(),
            self.shape.as_deref(),
            self.flat,
            self.flat_lists,
        );
    }
}

/// Apply pagination + shape + flat fields to a query-pair list. Used by every
/// resource module that flattens those fields onto its options builder.
pub(crate) fn apply_pagination(
    q: &mut Vec<(String, String)>,
    page: Option<u32>,
    limit: Option<u32>,
    cursor: Option<&str>,
    shape: Option<&str>,
    flat: bool,
    flat_lists: bool,
) {
    if let Some(c) = cursor.filter(|s| !s.is_empty()) {
        q.push(("cursor".into(), c.into()));
    } else if let Some(p) = page.filter(|p| *p > 0) {
        q.push(("page".into(), p.to_string()));
    }
    if let Some(l) = limit.filter(|l| *l > 0) {
        q.push(("limit".into(), l.to_string()));
    }
    if let Some(s) = shape.filter(|s| !s.is_empty()) {
        q.push(("shape".into(), s.into()));
    }
    if flat {
        q.push(("flat".into(), "true".into()));
    }
    if flat_lists {
        q.push(("flat_lists".into(), "true".into()));
    }
}

/// Push a `(key, value)` pair when `value` is `Some` and non-empty.
pub(crate) fn push_opt(q: &mut Vec<(String, String)>, key: &str, value: Option<&str>) {
    if let Some(v) = value.filter(|v| !v.is_empty()) {
        q.push((key.into(), v.into()));
    }
}

/// Push a `(key, value)` pair when `value` is `Some(true|false)`.
pub(crate) fn push_opt_bool(q: &mut Vec<(String, String)>, key: &str, value: Option<bool>) {
    if let Some(v) = value {
        q.push((key.into(), v.to_string()));
    }
}

/// Push a `(key, value)` pair when `value` is `Some(n)` and non-zero.
pub(crate) fn push_opt_u32(q: &mut Vec<(String, String)>, key: &str, value: Option<u32>) {
    if let Some(v) = value.filter(|n| *n > 0) {
        q.push((key.into(), v.to_string()));
    }
}

/// Pick the first non-empty string from a slice of optional borrows.
pub(crate) fn first_non_empty<'a>(values: &[Option<&'a str>]) -> Option<&'a str> {
    values.iter().copied().flatten().find(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_options_cursor_wins_over_page() {
        let opts = ListOptions::builder()
            .page(2u32)
            .cursor("abc".to_string())
            .build();
        let mut q = Vec::new();
        opts.apply(&mut q);
        assert!(q.contains(&("cursor".into(), "abc".into())));
        assert!(!q.iter().any(|(k, _)| k == "page"));
    }

    #[test]
    fn list_options_only_emits_set_fields() {
        let opts = ListOptions::builder().limit(25u32).build();
        let mut q = Vec::new();
        opts.apply(&mut q);
        assert_eq!(q, vec![("limit".to_string(), "25".to_string())]);
    }

    #[test]
    fn list_options_flat_emits_string() {
        let opts = ListOptions::builder().flat(true).flat_lists(true).build();
        let mut q = Vec::new();
        opts.apply(&mut q);
        assert!(q.contains(&("flat".into(), "true".into())));
        assert!(q.contains(&("flat_lists".into(), "true".into())));
    }

    #[test]
    fn first_non_empty_skips_empties() {
        assert_eq!(
            first_non_empty(&[Some(""), Some("a"), Some("b")]),
            Some("a")
        );
        assert_eq!(first_non_empty(&[None, None]), None);
        assert_eq!(first_non_empty(&[Some("")]), None);
    }
}
