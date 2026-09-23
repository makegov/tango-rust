//! `GET /api/dibbs/` — Defense Logistics Agency (DLA) DIBBS solicitations and awards: requests for quote (RFQs), requests for proposal (RFPs), and award line items.
//!
//! Every DIBBS record belongs to the Defense Logistics Agency.
//! Open/closed on RFQs and RFPs is derived at query time from the response date, so filter with `open` rather than comparing a returned `is_open` from an older page.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt, push_opt_bool};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_dibbs_rfqs`] and [`Client::iterate_dibbs_rfqs`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListDibbsRfqsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use [`SHAPE_DIBBS_RFQS_MINIMAL`](crate::SHAPE_DIBBS_RFQS_MINIMAL) or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Whether the RFQ is still accepting quotes as of today (its `return_by_date` has not passed). Derived at query time; there is no stored flag. `Some(false)` reaches the server as a filter value.
    pub open: Option<bool>,
    /// National Stock Number. Supports OR via `|`.
    #[builder(into)]
    pub nsn: Option<String>,
    /// Manufacturer part number. Supports OR via `|`.
    #[builder(into)]
    pub part_number: Option<String>,
    /// DLA solicitation number (e.g. `SPE1C126Q0337`).
    #[builder(into)]
    pub solicitation: Option<String>,
    /// Purchase request number. Supports OR via `|`.
    #[builder(into)]
    pub purchase_request: Option<String>,
    /// Set-aside flag (`Y` or `N`).
    #[builder(into)]
    pub set_aside: Option<String>,
    /// DIBBS status code. Supports OR via `|`.
    #[builder(into)]
    pub status_code: Option<String>,
    /// The resolved organization's federal hierarchy key.
    #[builder(into)]
    pub organization: Option<String>,
    /// Quantity greater than or equal to.
    #[builder(into)]
    pub quantity_min: Option<String>,
    /// Quantity less than or equal to.
    #[builder(into)]
    pub quantity_max: Option<String>,
    /// Return-by date on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub return_by_date_after: Option<String>,
    /// Return-by date on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub return_by_date_before: Option<String>,
    /// Issue date on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub issue_date_after: Option<String>,
    /// Issue date on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub issue_date_before: Option<String>,
    /// Full-text search across nomenclature, NSN, part number, and solicitation number.
    #[builder(into)]
    pub search: Option<String>,
    /// One of `issue_date`, `return_by_date`, `quantity`, `modified`, each optionally `-` prefixed.
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListDibbsRfqsOptions {
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
        push_opt_bool(&mut q, "open", self.open);
        push_opt(&mut q, "nsn", self.nsn.as_deref());
        push_opt(&mut q, "part_number", self.part_number.as_deref());
        push_opt(&mut q, "solicitation", self.solicitation.as_deref());
        push_opt(&mut q, "purchase_request", self.purchase_request.as_deref());
        push_opt(&mut q, "set_aside", self.set_aside.as_deref());
        push_opt(&mut q, "status_code", self.status_code.as_deref());
        push_opt(&mut q, "organization", self.organization.as_deref());
        push_opt(&mut q, "quantity_min", self.quantity_min.as_deref());
        push_opt(&mut q, "quantity_max", self.quantity_max.as_deref());
        push_opt(
            &mut q,
            "return_by_date_after",
            self.return_by_date_after.as_deref(),
        );
        push_opt(
            &mut q,
            "return_by_date_before",
            self.return_by_date_before.as_deref(),
        );
        push_opt(&mut q, "issue_date_after", self.issue_date_after.as_deref());
        push_opt(
            &mut q,
            "issue_date_before",
            self.issue_date_before.as_deref(),
        );
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

/// Options for [`Client::list_dibbs_rfps`] and [`Client::iterate_dibbs_rfps`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListDibbsRfpsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use [`SHAPE_DIBBS_RFPS_MINIMAL`](crate::SHAPE_DIBBS_RFPS_MINIMAL) or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Whether the solicitation is still accepting offers as of today (its `closes_date` has not passed). Derived at query time; there is no stored flag. `Some(false)` reaches the server as a filter value.
    pub open: Option<bool>,
    /// National Stock Number. Supports OR via `|`.
    #[builder(into)]
    pub nsn: Option<String>,
    /// Manufacturer part number. Supports OR via `|`.
    #[builder(into)]
    pub part_number: Option<String>,
    /// DLA solicitation number.
    #[builder(into)]
    pub solicitation: Option<String>,
    /// DLA buyer code. Supports OR via `|`.
    #[builder(into)]
    pub buyer_code: Option<String>,
    /// The resolved organization's federal hierarchy key.
    #[builder(into)]
    pub organization: Option<String>,
    /// Issued date on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub issued_date_after: Option<String>,
    /// Issued date on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub issued_date_before: Option<String>,
    /// Close date on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub closes_date_after: Option<String>,
    /// Close date on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub closes_date_before: Option<String>,
    /// Full-text search across nomenclature, NSN, part number, and solicitation number.
    #[builder(into)]
    pub search: Option<String>,
    /// One of `issued_date`, `closes_date`, `modified`, each optionally `-` prefixed.
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListDibbsRfpsOptions {
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
        push_opt_bool(&mut q, "open", self.open);
        push_opt(&mut q, "nsn", self.nsn.as_deref());
        push_opt(&mut q, "part_number", self.part_number.as_deref());
        push_opt(&mut q, "solicitation", self.solicitation.as_deref());
        push_opt(&mut q, "buyer_code", self.buyer_code.as_deref());
        push_opt(&mut q, "organization", self.organization.as_deref());
        push_opt(
            &mut q,
            "issued_date_after",
            self.issued_date_after.as_deref(),
        );
        push_opt(
            &mut q,
            "issued_date_before",
            self.issued_date_before.as_deref(),
        );
        push_opt(
            &mut q,
            "closes_date_after",
            self.closes_date_after.as_deref(),
        );
        push_opt(
            &mut q,
            "closes_date_before",
            self.closes_date_before.as_deref(),
        );
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

/// Options for [`Client::list_dibbs_awards`] and [`Client::iterate_dibbs_awards`].
///
/// Each row is one line item of an award. **`total_contract_price` is the order total repeated on every line item**, so never sum it across rows: deduplicate on `award_number` + `delivery_order_number` first.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListDibbsAwardsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use [`SHAPE_DIBBS_AWARDS_MINIMAL`](crate::SHAPE_DIBBS_AWARDS_MINIMAL) or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// National Stock Number. Supports OR via `|`.
    #[builder(into)]
    pub nsn: Option<String>,
    /// Manufacturer part number. Supports OR via `|`.
    #[builder(into)]
    pub part_number: Option<String>,
    /// DLA solicitation number.
    #[builder(into)]
    pub solicitation: Option<String>,
    /// DLA award (contract) number. Order-scoped: one award number spans every line item of the order.
    #[builder(into)]
    pub award_number: Option<String>,
    /// Delivery order number. Null on a basic award with no delivery order.
    #[builder(into)]
    pub delivery_order_number: Option<String>,
    /// Purchase request number. Supports OR via `|`.
    #[builder(into)]
    pub purchase_request: Option<String>,
    /// The awardee's CAGE code. Populated on every award row, whether or not it resolved to a Tango entity.
    #[builder(into)]
    pub awardee_cage: Option<String>,
    /// UEI of the resolved Tango entity. Set only when the awardee's CAGE code matched a registered entity, so an unresolved award never matches it; use [`awardee_cage`](Self::awardee_cage) to reach every award.
    #[builder(into)]
    pub entity: Option<String>,
    /// The resolved organization's federal hierarchy key.
    #[builder(into)]
    pub organization: Option<String>,
    /// Award date on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub award_date_after: Option<String>,
    /// Award date on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub award_date_before: Option<String>,
    /// Posted date on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub posted_date_after: Option<String>,
    /// Posted date on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub posted_date_before: Option<String>,
    /// Order total contract price greater than or equal to.
    #[builder(into)]
    pub total_contract_price_min: Option<String>,
    /// Order total contract price less than or equal to.
    #[builder(into)]
    pub total_contract_price_max: Option<String>,
    /// Full-text search across nomenclature, NSN, part number, and solicitation number.
    #[builder(into)]
    pub search: Option<String>,
    /// One of `award_date`, `posted_date`, `total_contract_price`, `modified`, each optionally `-` prefixed.
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListDibbsAwardsOptions {
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
        push_opt(&mut q, "nsn", self.nsn.as_deref());
        push_opt(&mut q, "part_number", self.part_number.as_deref());
        push_opt(&mut q, "solicitation", self.solicitation.as_deref());
        push_opt(&mut q, "award_number", self.award_number.as_deref());
        push_opt(
            &mut q,
            "delivery_order_number",
            self.delivery_order_number.as_deref(),
        );
        push_opt(&mut q, "purchase_request", self.purchase_request.as_deref());
        push_opt(&mut q, "awardee_cage", self.awardee_cage.as_deref());
        push_opt(&mut q, "entity", self.entity.as_deref());
        push_opt(&mut q, "organization", self.organization.as_deref());
        push_opt(&mut q, "award_date_after", self.award_date_after.as_deref());
        push_opt(
            &mut q,
            "award_date_before",
            self.award_date_before.as_deref(),
        );
        push_opt(
            &mut q,
            "posted_date_after",
            self.posted_date_after.as_deref(),
        );
        push_opt(
            &mut q,
            "posted_date_before",
            self.posted_date_before.as_deref(),
        );
        push_opt(
            &mut q,
            "total_contract_price_min",
            self.total_contract_price_min.as_deref(),
        );
        push_opt(
            &mut q,
            "total_contract_price_max",
            self.total_contract_price_max.as_deref(),
        );
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

/// Options for [`Client::get_dibbs_rfq`], [`Client::get_dibbs_rfp`], [`Client::get_dibbs_award`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetDibbsOptions {
    /// Shape selector. When empty, the server returns its default detail shape.
    #[builder(into)]
    pub shape: Option<String>,
    /// Flatten nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When `flat=true`, also flatten list-valued nested fields.
    #[builder(default)]
    pub flat_lists: bool,
}

impl GetDibbsOptions {
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
    /// `GET /api/dibbs/rfqs/` — one page of DIBBS request-for-quote line records.
    pub async fn list_dibbs_rfqs(&self, opts: ListDibbsRfqsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/dibbs/rfqs/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/dibbs/rfqs/{uuid}/` — a single DIBBS request-for-quote line.
    pub async fn get_dibbs_rfq(&self, uuid: &str, opts: Option<GetDibbsOptions>) -> Result<Record> {
        if uuid.is_empty() {
            return Err(Error::Validation {
                message: "get_dibbs_rfq: uuid is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/dibbs/rfqs/{}/", urlencoding(uuid));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every DIBBS request-for-quote line matching `opts`.
    pub fn iterate_dibbs_rfqs(&self, opts: ListDibbsRfqsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_dibbs_rfqs(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/dibbs/rfps/` — one page of DIBBS request-for-proposal records.
    pub async fn list_dibbs_rfps(&self, opts: ListDibbsRfpsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/dibbs/rfps/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/dibbs/rfps/{uuid}/` — a single DIBBS request-for-proposal.
    pub async fn get_dibbs_rfp(&self, uuid: &str, opts: Option<GetDibbsOptions>) -> Result<Record> {
        if uuid.is_empty() {
            return Err(Error::Validation {
                message: "get_dibbs_rfp: uuid is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/dibbs/rfps/{}/", urlencoding(uuid));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every DIBBS request-for-proposal matching `opts`.
    pub fn iterate_dibbs_rfps(&self, opts: ListDibbsRfpsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_dibbs_rfps(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/dibbs/awards/` — one page of DIBBS award line item records.
    pub async fn list_dibbs_awards(&self, opts: ListDibbsAwardsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/dibbs/awards/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/dibbs/awards/{uuid}/` — a single DIBBS award line item.
    pub async fn get_dibbs_award(
        &self,
        uuid: &str,
        opts: Option<GetDibbsOptions>,
    ) -> Result<Record> {
        if uuid.is_empty() {
            return Err(Error::Validation {
                message: "get_dibbs_award: uuid is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/dibbs/awards/{}/", urlencoding(uuid));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every DIBBS award line item matching `opts`.
    pub fn iterate_dibbs_awards(&self, opts: ListDibbsAwardsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_dibbs_awards(next).await })
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
    fn list_dibbs_rfqs_all_filters_emit() {
        let opts = ListDibbsRfqsOptions::builder()
            .open(true)
            .nsn("v-nsn")
            .part_number("v-part_number")
            .solicitation("v-solicitation")
            .purchase_request("v-purchase_request")
            .set_aside("v-set_aside")
            .status_code("v-status_code")
            .organization("v-organization")
            .quantity_min("v-quantity_min")
            .quantity_max("v-quantity_max")
            .return_by_date_after("v-return_by_date_after")
            .return_by_date_before("v-return_by_date_before")
            .issue_date_after("v-issue_date_after")
            .issue_date_before("v-issue_date_before")
            .search("v-search")
            .ordering("v-ordering")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "open").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "nsn").as_deref(), Some("v-nsn"));
        assert_eq!(get_q(&q, "part_number").as_deref(), Some("v-part_number"));
        assert_eq!(get_q(&q, "solicitation").as_deref(), Some("v-solicitation"));
        assert_eq!(
            get_q(&q, "purchase_request").as_deref(),
            Some("v-purchase_request")
        );
        assert_eq!(get_q(&q, "set_aside").as_deref(), Some("v-set_aside"));
        assert_eq!(get_q(&q, "status_code").as_deref(), Some("v-status_code"));
        assert_eq!(get_q(&q, "organization").as_deref(), Some("v-organization"));
        assert_eq!(get_q(&q, "quantity_min").as_deref(), Some("v-quantity_min"));
        assert_eq!(get_q(&q, "quantity_max").as_deref(), Some("v-quantity_max"));
        assert_eq!(
            get_q(&q, "return_by_date_after").as_deref(),
            Some("v-return_by_date_after")
        );
        assert_eq!(
            get_q(&q, "return_by_date_before").as_deref(),
            Some("v-return_by_date_before")
        );
        assert_eq!(
            get_q(&q, "issue_date_after").as_deref(),
            Some("v-issue_date_after")
        );
        assert_eq!(
            get_q(&q, "issue_date_before").as_deref(),
            Some("v-issue_date_before")
        );
        assert_eq!(get_q(&q, "search").as_deref(), Some("v-search"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("v-ordering"));
        assert_eq!(q.len(), 16);
    }

    #[test]
    fn list_dibbs_rfqs_open_false_is_a_filter_not_an_absence() {
        let opts = ListDibbsRfqsOptions::builder().open(false).build();
        assert_eq!(get_q(&opts.to_query(), "open").as_deref(), Some("false"));
    }

    #[test]
    fn list_dibbs_rfqs_zero_value_omitted() {
        assert!(ListDibbsRfqsOptions::builder()
            .build()
            .to_query()
            .is_empty());
    }

    #[test]
    fn list_dibbs_rfqs_cursor_wins_over_page_and_shape_emits() {
        let opts = ListDibbsRfqsOptions::builder()
            .page(3u32)
            .cursor("c0".to_string())
            .shape(crate::SHAPE_DIBBS_RFQS_MINIMAL)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("c0"));
        assert_eq!(get_q(&q, "page"), None);
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_DIBBS_RFQS_MINIMAL)
        );
    }

    #[test]
    fn list_dibbs_rfqs_extra_emits() {
        let mut extra = BTreeMap::new();
        extra.insert("custom_x".to_string(), "xv".to_string());
        let q = ListDibbsRfqsOptions::builder()
            .extra(extra)
            .build()
            .to_query();
        assert!(q.contains(&("custom_x".into(), "xv".into())));
    }

    #[tokio::test]
    async fn get_dibbs_rfq_validates_empty_uuid() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_dibbs_rfq("", None).await.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("uuid is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn list_dibbs_rfps_all_filters_emit() {
        let opts = ListDibbsRfpsOptions::builder()
            .open(true)
            .nsn("v-nsn")
            .part_number("v-part_number")
            .solicitation("v-solicitation")
            .buyer_code("v-buyer_code")
            .organization("v-organization")
            .issued_date_after("v-issued_date_after")
            .issued_date_before("v-issued_date_before")
            .closes_date_after("v-closes_date_after")
            .closes_date_before("v-closes_date_before")
            .search("v-search")
            .ordering("v-ordering")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "open").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "nsn").as_deref(), Some("v-nsn"));
        assert_eq!(get_q(&q, "part_number").as_deref(), Some("v-part_number"));
        assert_eq!(get_q(&q, "solicitation").as_deref(), Some("v-solicitation"));
        assert_eq!(get_q(&q, "buyer_code").as_deref(), Some("v-buyer_code"));
        assert_eq!(get_q(&q, "organization").as_deref(), Some("v-organization"));
        assert_eq!(
            get_q(&q, "issued_date_after").as_deref(),
            Some("v-issued_date_after")
        );
        assert_eq!(
            get_q(&q, "issued_date_before").as_deref(),
            Some("v-issued_date_before")
        );
        assert_eq!(
            get_q(&q, "closes_date_after").as_deref(),
            Some("v-closes_date_after")
        );
        assert_eq!(
            get_q(&q, "closes_date_before").as_deref(),
            Some("v-closes_date_before")
        );
        assert_eq!(get_q(&q, "search").as_deref(), Some("v-search"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("v-ordering"));
        assert_eq!(q.len(), 12);
    }

    #[test]
    fn list_dibbs_rfps_open_false_is_a_filter_not_an_absence() {
        let opts = ListDibbsRfpsOptions::builder().open(false).build();
        assert_eq!(get_q(&opts.to_query(), "open").as_deref(), Some("false"));
    }

    #[test]
    fn list_dibbs_rfps_zero_value_omitted() {
        assert!(ListDibbsRfpsOptions::builder()
            .build()
            .to_query()
            .is_empty());
    }

    #[test]
    fn list_dibbs_rfps_cursor_wins_over_page_and_shape_emits() {
        let opts = ListDibbsRfpsOptions::builder()
            .page(3u32)
            .cursor("c0".to_string())
            .shape(crate::SHAPE_DIBBS_RFPS_MINIMAL)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("c0"));
        assert_eq!(get_q(&q, "page"), None);
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_DIBBS_RFPS_MINIMAL)
        );
    }

    #[test]
    fn list_dibbs_rfps_extra_emits() {
        let mut extra = BTreeMap::new();
        extra.insert("custom_x".to_string(), "xv".to_string());
        let q = ListDibbsRfpsOptions::builder()
            .extra(extra)
            .build()
            .to_query();
        assert!(q.contains(&("custom_x".into(), "xv".into())));
    }

    #[tokio::test]
    async fn get_dibbs_rfp_validates_empty_uuid() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_dibbs_rfp("", None).await.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("uuid is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn list_dibbs_awards_all_filters_emit() {
        let opts = ListDibbsAwardsOptions::builder()
            .nsn("v-nsn")
            .part_number("v-part_number")
            .solicitation("v-solicitation")
            .award_number("v-award_number")
            .delivery_order_number("v-delivery_order_number")
            .purchase_request("v-purchase_request")
            .awardee_cage("v-awardee_cage")
            .entity("v-entity")
            .organization("v-organization")
            .award_date_after("v-award_date_after")
            .award_date_before("v-award_date_before")
            .posted_date_after("v-posted_date_after")
            .posted_date_before("v-posted_date_before")
            .total_contract_price_min("v-total_contract_price_min")
            .total_contract_price_max("v-total_contract_price_max")
            .search("v-search")
            .ordering("v-ordering")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "nsn").as_deref(), Some("v-nsn"));
        assert_eq!(get_q(&q, "part_number").as_deref(), Some("v-part_number"));
        assert_eq!(get_q(&q, "solicitation").as_deref(), Some("v-solicitation"));
        assert_eq!(get_q(&q, "award_number").as_deref(), Some("v-award_number"));
        assert_eq!(
            get_q(&q, "delivery_order_number").as_deref(),
            Some("v-delivery_order_number")
        );
        assert_eq!(
            get_q(&q, "purchase_request").as_deref(),
            Some("v-purchase_request")
        );
        assert_eq!(get_q(&q, "awardee_cage").as_deref(), Some("v-awardee_cage"));
        assert_eq!(get_q(&q, "entity").as_deref(), Some("v-entity"));
        assert_eq!(get_q(&q, "organization").as_deref(), Some("v-organization"));
        assert_eq!(
            get_q(&q, "award_date_after").as_deref(),
            Some("v-award_date_after")
        );
        assert_eq!(
            get_q(&q, "award_date_before").as_deref(),
            Some("v-award_date_before")
        );
        assert_eq!(
            get_q(&q, "posted_date_after").as_deref(),
            Some("v-posted_date_after")
        );
        assert_eq!(
            get_q(&q, "posted_date_before").as_deref(),
            Some("v-posted_date_before")
        );
        assert_eq!(
            get_q(&q, "total_contract_price_min").as_deref(),
            Some("v-total_contract_price_min")
        );
        assert_eq!(
            get_q(&q, "total_contract_price_max").as_deref(),
            Some("v-total_contract_price_max")
        );
        assert_eq!(get_q(&q, "search").as_deref(), Some("v-search"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("v-ordering"));
        assert_eq!(q.len(), 17);
    }

    #[test]
    fn list_dibbs_awards_zero_value_omitted() {
        assert!(ListDibbsAwardsOptions::builder()
            .build()
            .to_query()
            .is_empty());
    }

    #[test]
    fn list_dibbs_awards_cursor_wins_over_page_and_shape_emits() {
        let opts = ListDibbsAwardsOptions::builder()
            .page(3u32)
            .cursor("c0".to_string())
            .shape(crate::SHAPE_DIBBS_AWARDS_MINIMAL)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("c0"));
        assert_eq!(get_q(&q, "page"), None);
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_DIBBS_AWARDS_MINIMAL)
        );
    }

    #[test]
    fn list_dibbs_awards_extra_emits() {
        let mut extra = BTreeMap::new();
        extra.insert("custom_x".to_string(), "xv".to_string());
        let q = ListDibbsAwardsOptions::builder()
            .extra(extra)
            .build()
            .to_query();
        assert!(q.contains(&("custom_x".into(), "xv".into())));
    }

    #[tokio::test]
    async fn get_dibbs_award_validates_empty_uuid() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_dibbs_award("", None).await.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("uuid is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn get_options_emits() {
        let q = GetDibbsOptions::builder()
            .shape("a,b")
            .flat(true)
            .flat_lists(true)
            .build()
            .to_query();
        assert_eq!(get_q(&q, "shape").as_deref(), Some("a,b"));
        assert_eq!(get_q(&q, "flat").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "flat_lists").as_deref(), Some("true"));
    }
}
