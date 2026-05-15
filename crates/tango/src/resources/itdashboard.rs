//! `GET /api/itdashboard/` — federal IT investments from the
//! IT Dashboard.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_itdashboard`] and [`Client::iterate_itdashboard`].
///
/// Filters are tier-gated by the API:
/// - **Free:** [`search`](Self::search)
/// - **Pro:** [`agency_code`](Self::agency_code), [`type_of_investment`](Self::type_of_investment),
///   [`updated_time_after`](Self::updated_time_after), [`updated_time_before`](Self::updated_time_before)
/// - **Business+:** [`agency_name`](Self::agency_name), [`cio_rating`](Self::cio_rating),
///   [`cio_rating_max`](Self::cio_rating_max), [`performance_risk`](Self::performance_risk)
///
/// Hitting a gated filter on a lower tier returns a 403. CIO ratings:
/// 1=High Risk, 2=Moderately High, 3=Medium, 4=Moderately Low, 5=Low.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListItdashboardOptions {
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
    /// [`SHAPE_ITDASHBOARD_INVESTMENTS_MINIMAL`](crate::SHAPE_ITDASHBOARD_INVESTMENTS_MINIMAL)
    /// or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Free-text search (Free tier).
    #[builder(into)]
    pub search: Option<String>,
    /// Agency code filter (Pro tier).
    #[builder(into)]
    pub agency_code: Option<String>,
    /// Agency name filter (Business+ tier).
    #[builder(into)]
    pub agency_name: Option<String>,
    /// Investment type filter (Pro tier).
    #[builder(into)]
    pub type_of_investment: Option<String>,

    /// Lower bound on `updated_time` (Pro tier, ISO 8601).
    #[builder(into)]
    pub updated_time_after: Option<String>,
    /// Upper bound on `updated_time` (Pro tier, ISO 8601).
    #[builder(into)]
    pub updated_time_before: Option<String>,

    /// CIO rating filter (Business+ tier). Stringly typed to disambiguate
    /// "unset" from numeric zero (the API accepts both numeric and stringified
    /// integer values).
    #[builder(into)]
    pub cio_rating: Option<String>,
    /// Upper bound on CIO rating (Business+ tier).
    #[builder(into)]
    pub cio_rating_max: Option<String>,
    /// Performance-risk filter (Business+ tier).
    #[builder(into)]
    pub performance_risk: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListItdashboardOptions {
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
        push_opt(&mut q, "search", self.search.as_deref());
        push_opt(&mut q, "agency_code", self.agency_code.as_deref());
        push_opt(&mut q, "agency_name", self.agency_name.as_deref());
        push_opt(
            &mut q,
            "type_of_investment",
            self.type_of_investment.as_deref(),
        );
        push_opt(
            &mut q,
            "updated_time_after",
            self.updated_time_after.as_deref(),
        );
        push_opt(
            &mut q,
            "updated_time_before",
            self.updated_time_before.as_deref(),
        );
        push_opt(&mut q, "cio_rating", self.cio_rating.as_deref());
        push_opt(&mut q, "cio_rating_max", self.cio_rating_max.as_deref());
        push_opt(&mut q, "performance_risk", self.performance_risk.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

/// Options for [`Client::get_itdashboard`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetItdashboardOptions {
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

impl GetItdashboardOptions {
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
    /// `GET /api/itdashboard/` — one page of federal IT investments.
    pub async fn list_itdashboard(&self, opts: ListItdashboardOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/itdashboard/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/itdashboard/{uii}/` — fetch a single investment
    /// by Unique Investment Identifier (UII).
    pub async fn get_itdashboard(
        &self,
        uii: &str,
        opts: Option<GetItdashboardOptions>,
    ) -> Result<Record> {
        if uii.is_empty() {
            return Err(Error::Validation {
                message: "get_itdashboard: uii is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/itdashboard/{}/", urlencoding(uii));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every IT Dashboard investment matching `opts`.
    pub fn iterate_itdashboard(&self, opts: ListItdashboardOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_itdashboard(next).await })
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
    fn list_itdashboard_all_filters_emit() {
        let opts = ListItdashboardOptions::builder()
            .search("cloud")
            .agency_code("009")
            .agency_name("Department of State")
            .type_of_investment("Major")
            .updated_time_after("2024-01-01T00:00:00Z")
            .updated_time_before("2024-12-31T23:59:59Z")
            .cio_rating("3")
            .cio_rating_max("5")
            .performance_risk("2")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "search").as_deref(), Some("cloud"));
        assert_eq!(get_q(&q, "agency_code").as_deref(), Some("009"));
        assert_eq!(
            get_q(&q, "agency_name").as_deref(),
            Some("Department of State")
        );
        assert_eq!(get_q(&q, "type_of_investment").as_deref(), Some("Major"));
        assert_eq!(
            get_q(&q, "updated_time_after").as_deref(),
            Some("2024-01-01T00:00:00Z")
        );
        assert_eq!(
            get_q(&q, "updated_time_before").as_deref(),
            Some("2024-12-31T23:59:59Z")
        );
        assert_eq!(get_q(&q, "cio_rating").as_deref(), Some("3"));
        assert_eq!(get_q(&q, "cio_rating_max").as_deref(), Some("5"));
        assert_eq!(get_q(&q, "performance_risk").as_deref(), Some("2"));
    }

    #[test]
    fn list_itdashboard_zero_value_omitted() {
        let opts = ListItdashboardOptions::builder().build();
        let q = opts.to_query();
        assert!(q.is_empty(), "expected empty query, got {q:?}");
    }

    #[test]
    fn list_itdashboard_shape_emits() {
        let opts = ListItdashboardOptions::builder()
            .shape(crate::SHAPE_ITDASHBOARD_INVESTMENTS_MINIMAL)
            .build();
        let q = opts.to_query();
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_ITDASHBOARD_INVESTMENTS_MINIMAL)
        );
    }

    #[test]
    fn list_itdashboard_cursor_wins_over_page() {
        let opts = ListItdashboardOptions::builder()
            .page(3u32)
            .cursor("c0".to_string())
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("c0"));
        assert_eq!(get_q(&q, "page"), None);
    }

    #[test]
    fn list_itdashboard_extra_emits() {
        let mut extra = BTreeMap::new();
        extra.insert("custom_x".to_string(), "xv".to_string());
        let opts = ListItdashboardOptions::builder().extra(extra).build();
        let q = opts.to_query();
        assert!(q.contains(&("custom_x".into(), "xv".into())));
    }

    #[tokio::test]
    async fn get_itdashboard_validates_empty_uii() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_itdashboard("", None).await.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("uii is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
