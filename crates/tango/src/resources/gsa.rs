//! `GET /api/gsa_elibrary_contracts/` — GSA eLibrary contract listings.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_gsa_elibrary_contracts`] and
/// [`Client::iterate_gsa_elibrary_contracts`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListGsaElibraryContractsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use
    /// [`SHAPE_GSA_ELIBRARY_CONTRACTS_MINIMAL`](crate::SHAPE_GSA_ELIBRARY_CONTRACTS_MINIMAL)
    /// or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// GSA schedule filter (e.g. `"MAS"`).
    #[builder(into)]
    pub schedule: Option<String>,
    /// Contract-number filter.
    #[builder(into)]
    pub contract_number: Option<String>,
    /// Internal Tango key filter.
    #[builder(into)]
    pub key: Option<String>,
    /// Procurement Instrument Identifier filter.
    #[builder(into)]
    pub piid: Option<String>,
    /// Recipient UEI filter.
    #[builder(into)]
    pub uei: Option<String>,
    /// Special Item Number filter.
    #[builder(into)]
    pub sin: Option<String>,
    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// Server-side sort spec (prefix `-` for descending).
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListGsaElibraryContractsOptions {
    pub(crate) fn to_query(&self) -> Vec<(String, String)> {
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
        push_opt(&mut q, "schedule", self.schedule.as_deref());
        push_opt(&mut q, "contract_number", self.contract_number.as_deref());
        push_opt(&mut q, "key", self.key.as_deref());
        push_opt(&mut q, "piid", self.piid.as_deref());
        push_opt(&mut q, "uei", self.uei.as_deref());
        push_opt(&mut q, "sin", self.sin.as_deref());
        push_opt(&mut q, "search", self.search.as_deref());
        push_opt(&mut q, "ordering", self.ordering.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

/// Options for [`Client::get_gsa_elibrary_contract`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetGsaElibraryContractOptions {
    /// Shape selector. When empty, the server returns its comprehensive default.
    #[builder(into)]
    pub shape: Option<String>,
    /// Flatten nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When `flat=true`, also flatten list-valued nested fields.
    #[builder(default)]
    pub flat_lists: bool,
}

impl GetGsaElibraryContractOptions {
    pub(crate) fn to_query(&self) -> Vec<(String, String)> {
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
    /// `GET /api/gsa_elibrary_contracts/` — one page of GSA eLibrary contracts.
    pub async fn list_gsa_elibrary_contracts(
        &self,
        opts: ListGsaElibraryContractsOptions,
    ) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/gsa_elibrary_contracts/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/gsa_elibrary_contracts/{uuid}/` — fetch a single GSA eLibrary
    /// contract by UUID.
    pub async fn get_gsa_elibrary_contract(
        &self,
        uuid: &str,
        opts: Option<GetGsaElibraryContractOptions>,
    ) -> Result<Record> {
        if uuid.is_empty() {
            return Err(Error::Validation {
                message: "get_gsa_elibrary_contract: uuid is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/gsa_elibrary_contracts/{}/", urlencoding(uuid));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every GSA eLibrary contract matching `opts`.
    pub fn iterate_gsa_elibrary_contracts(
        &self,
        opts: ListGsaElibraryContractsOptions,
    ) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_gsa_elibrary_contracts(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_q(q: &[(String, String)], k: &str) -> Option<String> {
        q.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.clone())
    }

    #[test]
    fn list_gsa_all_filters_emit() {
        let opts = ListGsaElibraryContractsOptions::builder()
            .schedule("MAS")
            .contract_number("GS-35F-0119Y")
            .key("internal-key")
            .piid("GS35F0119Y")
            .uei("UEI12345")
            .sin("54151S")
            .search("cyber")
            .ordering("-contract_number")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "schedule").as_deref(), Some("MAS"));
        assert_eq!(
            get_q(&q, "contract_number").as_deref(),
            Some("GS-35F-0119Y")
        );
        assert_eq!(get_q(&q, "key").as_deref(), Some("internal-key"));
        assert_eq!(get_q(&q, "piid").as_deref(), Some("GS35F0119Y"));
        assert_eq!(get_q(&q, "uei").as_deref(), Some("UEI12345"));
        assert_eq!(get_q(&q, "sin").as_deref(), Some("54151S"));
        assert_eq!(get_q(&q, "search").as_deref(), Some("cyber"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("-contract_number"));
    }

    #[test]
    fn list_gsa_zero_value_omitted() {
        let opts = ListGsaElibraryContractsOptions::builder().build();
        let q = opts.to_query();
        assert!(q.is_empty(), "expected empty query, got {q:?}");
    }

    #[test]
    fn list_gsa_shape_emits() {
        let opts = ListGsaElibraryContractsOptions::builder()
            .shape(crate::SHAPE_GSA_ELIBRARY_CONTRACTS_MINIMAL)
            .build();
        let q = opts.to_query();
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_GSA_ELIBRARY_CONTRACTS_MINIMAL)
        );
    }

    #[test]
    fn list_gsa_cursor_wins_over_page() {
        let opts = ListGsaElibraryContractsOptions::builder()
            .page(4u32)
            .cursor("c0".to_string())
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("c0"));
        assert_eq!(get_q(&q, "page"), None);
    }

    #[test]
    fn list_gsa_extra_emits() {
        let mut extra = BTreeMap::new();
        extra.insert("custom_x".to_string(), "xv".to_string());
        let opts = ListGsaElibraryContractsOptions::builder()
            .extra(extra)
            .build();
        let q = opts.to_query();
        assert!(q.contains(&("custom_x".into(), "xv".into())));
    }

    #[tokio::test]
    async fn get_gsa_validates_empty_uuid() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client
            .get_gsa_elibrary_contract("", None)
            .await
            .unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("uuid is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
