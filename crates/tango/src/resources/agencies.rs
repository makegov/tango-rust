//! `GET /api/agencies/` and `GET /api/agencies/{code}/contracts/{which}/`.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt, push_opt_u32};
use crate::models::AgencyRecord;
use crate::pagination::{FetchFn, Page, PageStream};
use crate::Record;
use bon::Builder;
use std::sync::Arc;

/// Options for [`Client::list_agencies`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListAgenciesOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
}

impl ListAgenciesOptions {
    fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        push_opt_u32(&mut q, "page", self.page);
        if let Some(limit) = self.limit.filter(|n| *n > 0) {
            q.push(("limit".into(), limit.min(100).to_string()));
        }
        push_opt(&mut q, "search", self.search.as_deref());
        q
    }
}

/// Options for the agency contract sub-resources
/// (`list_agency_awarding_contracts` / `list_agency_funding_contracts`).
///
/// Mirrors `AgencyContractsOptions` in the Node SDK.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct AgencyContractsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor (mutually exclusive with page).
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
    /// Joiner used between flattened keys when `flat=true`. Defaults to `.` server-side.
    #[builder(into)]
    pub joiner: Option<String>,
    /// Server-side sort spec (endpoint-specific allowlist; prefix with `-` for descending).
    #[builder(into)]
    pub ordering: Option<String>,
    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
}

impl AgencyContractsOptions {
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
        if self.flat {
            if let Some(j) = self.joiner.as_deref().filter(|s| !s.is_empty()) {
                q.push(("joiner".into(), j.into()));
            }
        }
        push_opt(&mut q, "ordering", self.ordering.as_deref());
        push_opt(&mut q, "search", self.search.as_deref());
        q
    }
}

/// Alias matching the plan's `R-01` naming — same as [`AgencyContractsOptions`].
pub type ListAgencyAwardingContractsOptions = AgencyContractsOptions;
/// Alias matching the plan's `R-01` naming — same as [`AgencyContractsOptions`].
pub type ListAgencyFundingContractsOptions = AgencyContractsOptions;

/// Per-agency get options.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetAgencyOptions {
    /// Shape selector.
    #[builder(into)]
    pub shape: Option<String>,
    /// Flatten nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When `flat=true`, also flatten list-valued nested fields.
    #[builder(default)]
    pub flat_lists: bool,
}

impl GetAgencyOptions {
    fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        push_opt(&mut q, "shape", self.shape.as_deref());
        if self.flat {
            q.push(("flat".into(), "true".into()));
        }
        if self.flat_lists {
            q.push(("flat_lists".into(), "true".into()));
        }
        q
    }
}

impl Client {
    /// `GET /api/agencies/` — list federal agencies.
    pub async fn list_agencies(&self, opts: ListAgenciesOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/agencies/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/agencies/{code}/` — fetch a single agency.
    ///
    /// `code` is typically a CGAC code (e.g. `"9700"` for the Department of
    /// Defense). Returns a typed [`AgencyRecord`]; forward-compatible server
    /// fields land in [`AgencyRecord::extra`].
    pub async fn get_agency(
        &self,
        code: &str,
        opts: Option<GetAgencyOptions>,
    ) -> Result<AgencyRecord> {
        if code.is_empty() {
            return Err(Error::Validation {
                message: "agency code is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/agencies/{}/", urlencoding(code));
        self.get_json::<AgencyRecord>(&path, &q).await
    }

    /// `GET /api/agencies/{code}/contracts/awarding/` — contracts where this
    /// agency is the **awarding** agency.
    pub async fn list_agency_awarding_contracts(
        &self,
        code: &str,
        opts: AgencyContractsOptions,
    ) -> Result<Page<Record>> {
        list_agency_contracts(self, code, "awarding", opts).await
    }

    /// `GET /api/agencies/{code}/contracts/funding/` — contracts where this
    /// agency is the **funding** agency.
    pub async fn list_agency_funding_contracts(
        &self,
        code: &str,
        opts: AgencyContractsOptions,
    ) -> Result<Page<Record>> {
        list_agency_contracts(self, code, "funding", opts).await
    }

    /// Stream every contract where `code` is the awarding agency.
    pub fn iterate_agency_awarding_contracts(
        &self,
        code: &str,
        opts: AgencyContractsOptions,
    ) -> PageStream<Record> {
        iterate_agency_contracts(self, code.to_string(), "awarding", opts)
    }

    /// Stream every contract where `code` is the funding agency.
    pub fn iterate_agency_funding_contracts(
        &self,
        code: &str,
        opts: AgencyContractsOptions,
    ) -> PageStream<Record> {
        iterate_agency_contracts(self, code.to_string(), "funding", opts)
    }
}

async fn list_agency_contracts(
    client: &Client,
    code: &str,
    which: &str,
    opts: AgencyContractsOptions,
) -> Result<Page<Record>> {
    if code.is_empty() {
        return Err(Error::Validation {
            message: "agency code is required".into(),
            response: None,
        });
    }
    let q = opts.to_query();
    let path = format!("/api/agencies/{}/contracts/{which}/", urlencoding(code));
    let bytes = client.get_bytes(&path, &q).await?;
    Page::decode(&bytes)
}

fn iterate_agency_contracts(
    client: &Client,
    code: String,
    which: &'static str,
    opts: AgencyContractsOptions,
) -> PageStream<Record> {
    let opts = Arc::new(opts);
    let code = Arc::new(code);
    let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
        let opts = (*opts).clone();
        let mut next = opts;
        next.page = page;
        next.cursor = cursor;
        let code = code.clone();
        Box::pin(async move { list_agency_contracts(&client, &code, which, next).await })
    });
    PageStream::new(client.clone(), fetch)
}

/// Minimal path-segment escaping. The Tango API rejects `+` as a space
/// substitute, so we percent-encode like `url::PathEscape` (using `%20`).
pub(crate) fn urlencoding(s: &str) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => write!(&mut out, "%{byte:02X}").expect("write to String never fails"),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_agencies_caps_limit_at_100() {
        let opts = ListAgenciesOptions::builder().limit(500u32).build();
        let q = opts.to_query();
        assert_eq!(
            q.iter()
                .find(|(k, _)| k == "limit")
                .map(|(_, v)| v.as_str()),
            Some("100")
        );
    }

    #[test]
    fn list_agencies_emits_search() {
        let opts = ListAgenciesOptions::builder().search("defense").build();
        let q = opts.to_query();
        assert!(q.contains(&("search".into(), "defense".into())));
    }

    #[test]
    fn agency_contracts_emits_joiner_only_when_flat() {
        let opts = AgencyContractsOptions::builder()
            .joiner("__".to_string())
            .build();
        let q = opts.to_query();
        assert!(!q.iter().any(|(k, _)| k == "joiner"));

        let opts = AgencyContractsOptions::builder()
            .flat(true)
            .joiner("__".to_string())
            .build();
        let q = opts.to_query();
        assert!(q.contains(&("joiner".into(), "__".into())));
    }

    #[test]
    fn urlencoding_handles_special_chars() {
        assert_eq!(urlencoding("9700"), "9700");
        assert_eq!(urlencoding("AB CD"), "AB%20CD");
        assert_eq!(urlencoding("a/b"), "a%2Fb");
        assert_eq!(urlencoding("foo+bar"), "foo%2Bbar");
    }
}
