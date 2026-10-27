//! `GET /api/assistance/{grants,loans,direct_payments,other}/` — federal financial-assistance awards from USAspending (Tango API 5.12.0).
//!
//! Four endpoints split the awards by the GSDM assistance type of each award's latest action, and all four take the same filters, shapes and sub-routes:
//!
//! | Methods | Endpoint | Assistance types |
//! | ------- | -------- | ---------------- |
//! | `*_assistance_grant*` | `/api/assistance/grants/` | `F001` grant, `F002` cooperative agreement |
//! | `*_assistance_loan*` | `/api/assistance/loans/` | `F003` direct loan, `F004` loan guarantee |
//! | `*_assistance_direct_payment*` | `/api/assistance/direct_payments/` | `F006`, `F007` direct payments |
//! | `*_assistance_other*` | `/api/assistance/other/` | `F005` insurance, `F008`–`F010` |
//!
//! Grant AWARDS are money already obligated to a recipient; grant OPPORTUNITIES you can apply for are [`Client::list_grants`](crate::Client::list_grants).
//! An award lives under one endpoint, and moves (same `key`, old URL 404s) when its latest action's type changes group.
//! A retired key from an award's `former_award_keys` still resolves on detail and sub-routes, to the award that carries it now.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt};
use crate::models::{AssistanceAwardListing, AssistanceAwardRecord, AssistanceTransaction};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

const GRANTS: &str = "grants";
const LOANS: &str = "loans";
const DIRECT_PAYMENTS: &str = "direct_payments";
const OTHER: &str = "other";

/// Options for the four assistance award lists: [`Client::list_assistance_grants`], [`Client::list_assistance_loans`], [`Client::list_assistance_direct_payments`], [`Client::list_assistance_other`] and their `iterate_*` streams.
///
/// Every field is sent under the API's own query-parameter name.
/// Filters marked "OR via `|`" match any of several values joined by `|`.
/// The lists are cursor-paginated only, so there is no `page`.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListAssistanceAwardsOptions {
    /// Page size (server default 25).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor. Pass the `cursor` field from the previous [`Page`](crate::Page).
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Unset, the server returns [`SHAPE_ASSISTANCE_AWARDS_MINIMAL`](crate::SHAPE_ASSISTANCE_AWARDS_MINIMAL).
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys. Requires [`shape`](Self::shape).
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,
    /// Joiner for flattened keys (only sent when `flat=true`; server default `.`).
    #[builder(into)]
    pub joiner: Option<String>,

    /// Award key (exact). OR via `|`.
    #[builder(into)]
    pub key: Option<String>,
    /// Federal Award Identification Number (exact). OR via `|`.
    #[builder(into)]
    pub fain: Option<String>,
    /// Unique Record Identifier, carried by aggregate records (exact). OR via `|`.
    #[builder(into)]
    pub uri: Option<String>,
    /// Funding opportunity number the award was made under, as on the Grants.gov opportunity (exact). OR via `|`.
    #[builder(into)]
    pub funding_opportunity_number: Option<String>,

    /// Recipient UEI (exact). OR via `|`.
    #[builder(into)]
    pub uei: Option<String>,
    /// Recipient name or UEI, matched against the recipients closest to it by name; a term under three characters is a 400. OR via `|` only (the word `OR` is not a separator here).
    #[builder(into)]
    pub recipient: Option<String>,

    /// Awarding agency: code, name or abbreviation, best-effort. OR via `|`.
    #[builder(into)]
    pub awarding_agency: Option<String>,
    /// Funding agency or department, matched like [`awarding_agency`](Self::awarding_agency). OR via `|`.
    #[builder(into)]
    pub funding_agency: Option<String>,

    /// The award's PRIMARY assistance listing (CFDA) number, e.g. `93.778`. OR via `|`.
    #[builder(into)]
    pub cfda_number: Option<String>,
    /// GSDM assistance type code (`F001`–`F010`), case-insensitive. OR via `|`. A code outside the endpoint's group matches nothing.
    #[builder(into)]
    pub assistance_type: Option<String>,
    /// Record type: `1` aggregate, `2` non-aggregate, `3` individual recipient (PII-redacted). OR via `|`.
    #[builder(into)]
    pub record_type: Option<String>,
    /// Business funds indicator: `NON` (not Recovery Act) or `REC` (Recovery Act), case-insensitive.
    #[builder(into)]
    pub business_funds_indicator: Option<String>,
    /// Disaster Emergency Fund Code, e.g. `L` for COVID-19, matched as a whole code within the award's list. OR via `|`.
    #[builder(into)]
    pub disaster_emergency_fund_code: Option<String>,

    /// Award date (exact, `YYYY-MM-DD`).
    #[builder(into)]
    pub award_date: Option<String>,
    /// Lower bound for `award_date` (inclusive).
    #[builder(into)]
    pub award_date_gte: Option<String>,
    /// Upper bound for `award_date` (inclusive).
    #[builder(into)]
    pub award_date_lte: Option<String>,
    /// Federal fiscal year of the award (exact, `YYYY`).
    #[builder(into)]
    pub fiscal_year: Option<String>,
    /// Lower bound for `fiscal_year` (`YYYY`).
    #[builder(into)]
    pub fiscal_year_gte: Option<String>,
    /// Upper bound for `fiscal_year` (`YYYY`).
    #[builder(into)]
    pub fiscal_year_lte: Option<String>,
    /// Lower bound for the period-of-performance start date.
    #[builder(into)]
    pub pop_start_date_gte: Option<String>,
    /// Upper bound for the period-of-performance start date.
    #[builder(into)]
    pub pop_start_date_lte: Option<String>,
    /// Lower bound for the period-of-performance current end date.
    #[builder(into)]
    pub pop_end_date_gte: Option<String>,
    /// Upper bound for the period-of-performance current end date.
    #[builder(into)]
    pub pop_end_date_lte: Option<String>,

    /// Place-of-performance state code, e.g. `VA`, case-insensitive. OR via `|`.
    #[builder(into)]
    pub pop_state: Option<String>,
    /// Place-of-performance country code, ISO 3166 alpha-3, e.g. `USA`, case-insensitive. OR via `|`.
    #[builder(into)]
    pub pop_country: Option<String>,
    /// Place-of-performance five-digit ZIP code (exact). OR via `|`.
    #[builder(into)]
    pub pop_zip: Option<String>,

    /// Lower bound for `obligated` (USD).
    #[builder(into)]
    pub obligated_gte: Option<String>,
    /// Upper bound for `obligated` (USD).
    #[builder(into)]
    pub obligated_lte: Option<String>,

    /// Full-text search over identifiers, recipient names, assistance listing titles and descriptions.
    #[builder(into)]
    pub search: Option<String>,
    /// One of `award_date`, `obligated` or `pragmatic_obligations`, prefix `-` for descending. The server defaults to `-award_date`.
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for parameters not first-classed here, e.g. `exact=true` for an exact `count`.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListAssistanceAwardsOptions {
    pub(crate) fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        apply_pagination(
            &mut q,
            None,
            self.limit,
            self.cursor.as_deref(),
            self.shape.as_deref(),
            self.flat,
            self.flat_lists,
        );
        if self.flat {
            push_opt(&mut q, "joiner", self.joiner.as_deref());
        }
        for (name, value) in [
            ("key", &self.key),
            ("fain", &self.fain),
            ("uri", &self.uri),
            (
                "funding_opportunity_number",
                &self.funding_opportunity_number,
            ),
            ("uei", &self.uei),
            ("recipient", &self.recipient),
            ("awarding_agency", &self.awarding_agency),
            ("funding_agency", &self.funding_agency),
            ("cfda_number", &self.cfda_number),
            ("assistance_type", &self.assistance_type),
            ("record_type", &self.record_type),
            ("business_funds_indicator", &self.business_funds_indicator),
            (
                "disaster_emergency_fund_code",
                &self.disaster_emergency_fund_code,
            ),
            ("award_date", &self.award_date),
            ("award_date_gte", &self.award_date_gte),
            ("award_date_lte", &self.award_date_lte),
            ("fiscal_year", &self.fiscal_year),
            ("fiscal_year_gte", &self.fiscal_year_gte),
            ("fiscal_year_lte", &self.fiscal_year_lte),
            ("pop_start_date_gte", &self.pop_start_date_gte),
            ("pop_start_date_lte", &self.pop_start_date_lte),
            ("pop_end_date_gte", &self.pop_end_date_gte),
            ("pop_end_date_lte", &self.pop_end_date_lte),
            ("pop_state", &self.pop_state),
            ("pop_country", &self.pop_country),
            ("pop_zip", &self.pop_zip),
            ("obligated_gte", &self.obligated_gte),
            ("obligated_lte", &self.obligated_lte),
            ("search", &self.search),
            ("ordering", &self.ordering),
        ] {
            push_opt(&mut q, name, value.as_deref());
        }
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

/// Options for [`Client::get_assistance_grant`], [`Client::get_assistance_loan`], [`Client::get_assistance_direct_payment`] and [`Client::get_assistance_other`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetAssistanceAwardOptions {
    /// Shape selector. Unset, the server returns [`SHAPE_ASSISTANCE_AWARDS_MINIMAL`](crate::SHAPE_ASSISTANCE_AWARDS_MINIMAL).
    #[builder(into)]
    pub shape: Option<String>,
    /// Flatten nested objects into dot-separated keys. Flattened keys do not match [`AssistanceAwardRecord`]'s fields and land in its `extra`.
    #[builder(default)]
    pub flat: bool,
    /// When `flat=true`, also flatten list-valued nested fields.
    #[builder(default)]
    pub flat_lists: bool,
    /// Joiner for flattened keys (only sent when `flat=true`).
    #[builder(into)]
    pub joiner: Option<String>,
}

impl GetAssistanceAwardOptions {
    pub(crate) fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        apply_pagination(
            &mut q,
            None,
            None,
            None,
            self.shape.as_deref(),
            self.flat,
            self.flat_lists,
        );
        if self.flat {
            push_opt(&mut q, "joiner", self.joiner.as_deref());
        }
        q
    }
}

/// Options for an assistance award's `/transactions/` and `/listings/` sub-routes.
///
/// The sub-routes are page-numbered and take nothing else: no `shape`, filters or `ordering`.
/// `/transactions/` defaults to 100 rows a page (max 500), `/listings/` to 10 (max 100).
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct AssistanceSubresourceOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size.
    #[builder(into)]
    pub limit: Option<u32>,
}

impl AssistanceSubresourceOptions {
    pub(crate) fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        apply_pagination(&mut q, self.page, self.limit, None, None, false, false);
        q
    }
}

fn require_key(method: &str, key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(Error::Validation {
            message: format!("{method}: key is required"),
            response: None,
        });
    }
    Ok(())
}

impl Client {
    async fn list_assistance_awards(
        &self,
        group: &str,
        opts: ListAssistanceAwardsOptions,
    ) -> Result<Page<Record>> {
        let q = opts.to_query();
        self.get_page(&format!("/api/assistance/{group}/"), &q)
            .await
    }

    fn iterate_assistance_awards(
        &self,
        group: &'static str,
        opts: ListAssistanceAwardsOptions,
    ) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, _page, cursor| {
            let mut next = (*opts).clone();
            next.cursor = cursor;
            Box::pin(async move { client.list_assistance_awards(group, next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    async fn get_assistance_award(
        &self,
        method: &str,
        group: &str,
        key: &str,
        opts: Option<GetAssistanceAwardOptions>,
    ) -> Result<AssistanceAwardRecord> {
        require_key(method, key)?;
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/assistance/{group}/{}/", urlencoding(key));
        self.get_json::<AssistanceAwardRecord>(&path, &q).await
    }

    async fn list_assistance_award_subresource<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        group: &str,
        key: &str,
        sub: &str,
        opts: Option<AssistanceSubresourceOptions>,
    ) -> Result<Page<T>> {
        require_key(method, key)?;
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/assistance/{group}/{}/{sub}/", urlencoding(key));
        self.get_page(&path, &q).await
    }

    // ----- Grants and cooperative agreements -----

    /// `GET /api/assistance/grants/` — one page of grant and cooperative-agreement awards (`F001`, `F002`).
    ///
    /// Decode a row with [`AssistanceAwardRecord::from_record`].
    pub async fn list_assistance_grants(
        &self,
        opts: ListAssistanceAwardsOptions,
    ) -> Result<Page<Record>> {
        self.list_assistance_awards(GRANTS, opts).await
    }

    /// Stream every grant award matching `opts`, following the server's cursor.
    ///
    /// ```no_run
    /// # use tango::{Client, ListAssistanceAwardsOptions};
    /// # use futures::TryStreamExt;
    /// # async fn run(client: Client) -> tango::Result<()> {
    /// let mut s = client.iterate_assistance_grants(
    ///     ListAssistanceAwardsOptions::builder()
    ///         .cfda_number("93.778")
    ///         .ordering("-obligated")
    ///         .build(),
    /// );
    /// while let Some(record) = s.try_next().await? {
    ///     let _ = record;
    /// }
    /// # Ok(()) }
    /// ```
    pub fn iterate_assistance_grants(
        &self,
        opts: ListAssistanceAwardsOptions,
    ) -> PageStream<Record> {
        self.iterate_assistance_awards(GRANTS, opts)
    }

    /// `GET /api/assistance/grants/{key}/` — a single grant award. A retired key resolves to the award that carries it now.
    pub async fn get_assistance_grant(
        &self,
        key: &str,
        opts: Option<GetAssistanceAwardOptions>,
    ) -> Result<AssistanceAwardRecord> {
        self.get_assistance_award("get_assistance_grant", GRANTS, key, opts)
            .await
    }

    /// `GET /api/assistance/grants/{key}/transactions/` — a grant award's transactions, latest first; empty when the key names no award in the group.
    pub async fn list_assistance_grant_transactions(
        &self,
        key: &str,
        opts: Option<AssistanceSubresourceOptions>,
    ) -> Result<Page<AssistanceTransaction>> {
        self.list_assistance_award_subresource(
            "list_assistance_grant_transactions",
            GRANTS,
            key,
            "transactions",
            opts,
        )
        .await
    }

    /// `GET /api/assistance/grants/{key}/listings/` — a grant award's money by assistance listing (CFDA), primary first; empty when the key names no award in the group.
    pub async fn list_assistance_grant_listings(
        &self,
        key: &str,
        opts: Option<AssistanceSubresourceOptions>,
    ) -> Result<Page<AssistanceAwardListing>> {
        self.list_assistance_award_subresource(
            "list_assistance_grant_listings",
            GRANTS,
            key,
            "listings",
            opts,
        )
        .await
    }

    // ----- Loans -----

    /// `GET /api/assistance/loans/` — one page of direct-loan and loan-guarantee awards (`F003`, `F004`).
    ///
    /// On a loan `pragmatic_obligations` is the subsidy cost; `face_value_of_loan` is the principal.
    pub async fn list_assistance_loans(
        &self,
        opts: ListAssistanceAwardsOptions,
    ) -> Result<Page<Record>> {
        self.list_assistance_awards(LOANS, opts).await
    }

    /// Stream every loan award matching `opts`, following the server's cursor.
    pub fn iterate_assistance_loans(
        &self,
        opts: ListAssistanceAwardsOptions,
    ) -> PageStream<Record> {
        self.iterate_assistance_awards(LOANS, opts)
    }

    /// `GET /api/assistance/loans/{key}/` — a single loan award. A retired key resolves to the award that carries it now.
    pub async fn get_assistance_loan(
        &self,
        key: &str,
        opts: Option<GetAssistanceAwardOptions>,
    ) -> Result<AssistanceAwardRecord> {
        self.get_assistance_award("get_assistance_loan", LOANS, key, opts)
            .await
    }

    /// `GET /api/assistance/loans/{key}/transactions/` — a loan award's transactions, latest first; empty when the key names no award in the group.
    pub async fn list_assistance_loan_transactions(
        &self,
        key: &str,
        opts: Option<AssistanceSubresourceOptions>,
    ) -> Result<Page<AssistanceTransaction>> {
        self.list_assistance_award_subresource(
            "list_assistance_loan_transactions",
            LOANS,
            key,
            "transactions",
            opts,
        )
        .await
    }

    /// `GET /api/assistance/loans/{key}/listings/` — a loan award's money by assistance listing (CFDA), primary first; empty when the key names no award in the group.
    pub async fn list_assistance_loan_listings(
        &self,
        key: &str,
        opts: Option<AssistanceSubresourceOptions>,
    ) -> Result<Page<AssistanceAwardListing>> {
        self.list_assistance_award_subresource(
            "list_assistance_loan_listings",
            LOANS,
            key,
            "listings",
            opts,
        )
        .await
    }

    // ----- Direct payments -----

    /// `GET /api/assistance/direct_payments/` — one page of direct-payment awards (`F006`, `F007`).
    pub async fn list_assistance_direct_payments(
        &self,
        opts: ListAssistanceAwardsOptions,
    ) -> Result<Page<Record>> {
        self.list_assistance_awards(DIRECT_PAYMENTS, opts).await
    }

    /// Stream every direct-payment award matching `opts`, following the server's cursor.
    pub fn iterate_assistance_direct_payments(
        &self,
        opts: ListAssistanceAwardsOptions,
    ) -> PageStream<Record> {
        self.iterate_assistance_awards(DIRECT_PAYMENTS, opts)
    }

    /// `GET /api/assistance/direct_payments/{key}/` — a single direct-payment award. A retired key resolves to the award that carries it now.
    pub async fn get_assistance_direct_payment(
        &self,
        key: &str,
        opts: Option<GetAssistanceAwardOptions>,
    ) -> Result<AssistanceAwardRecord> {
        self.get_assistance_award("get_assistance_direct_payment", DIRECT_PAYMENTS, key, opts)
            .await
    }

    /// `GET /api/assistance/direct_payments/{key}/transactions/` — a direct-payment award's transactions, latest first; empty when the key names no award in the group.
    pub async fn list_assistance_direct_payment_transactions(
        &self,
        key: &str,
        opts: Option<AssistanceSubresourceOptions>,
    ) -> Result<Page<AssistanceTransaction>> {
        self.list_assistance_award_subresource(
            "list_assistance_direct_payment_transactions",
            DIRECT_PAYMENTS,
            key,
            "transactions",
            opts,
        )
        .await
    }

    /// `GET /api/assistance/direct_payments/{key}/listings/` — a direct-payment award's money by assistance listing (CFDA), primary first; empty when the key names no award in the group.
    pub async fn list_assistance_direct_payment_listings(
        &self,
        key: &str,
        opts: Option<AssistanceSubresourceOptions>,
    ) -> Result<Page<AssistanceAwardListing>> {
        self.list_assistance_award_subresource(
            "list_assistance_direct_payment_listings",
            DIRECT_PAYMENTS,
            key,
            "listings",
            opts,
        )
        .await
    }

    // ----- Other assistance -----

    /// `GET /api/assistance/other/` — one page of insurance, asset-forfeiture, property-sale and other assistance awards (`F005`, `F008`–`F010`).
    pub async fn list_assistance_other(
        &self,
        opts: ListAssistanceAwardsOptions,
    ) -> Result<Page<Record>> {
        self.list_assistance_awards(OTHER, opts).await
    }

    /// Stream every other-assistance award matching `opts`, following the server's cursor.
    pub fn iterate_assistance_other(
        &self,
        opts: ListAssistanceAwardsOptions,
    ) -> PageStream<Record> {
        self.iterate_assistance_awards(OTHER, opts)
    }

    /// `GET /api/assistance/other/{key}/` — a single other-assistance award. A retired key resolves to the award that carries it now.
    pub async fn get_assistance_other(
        &self,
        key: &str,
        opts: Option<GetAssistanceAwardOptions>,
    ) -> Result<AssistanceAwardRecord> {
        self.get_assistance_award("get_assistance_other", OTHER, key, opts)
            .await
    }

    /// `GET /api/assistance/other/{key}/transactions/` — an other-assistance award's transactions, latest first; empty when the key names no award in the group.
    pub async fn list_assistance_other_transactions(
        &self,
        key: &str,
        opts: Option<AssistanceSubresourceOptions>,
    ) -> Result<Page<AssistanceTransaction>> {
        self.list_assistance_award_subresource(
            "list_assistance_other_transactions",
            OTHER,
            key,
            "transactions",
            opts,
        )
        .await
    }

    /// `GET /api/assistance/other/{key}/listings/` — an other-assistance award's money by assistance listing (CFDA), primary first; empty when the key names no award in the group.
    pub async fn list_assistance_other_listings(
        &self,
        key: &str,
        opts: Option<AssistanceSubresourceOptions>,
    ) -> Result<Page<AssistanceAwardListing>> {
        self.list_assistance_award_subresource(
            "list_assistance_other_listings",
            OTHER,
            key,
            "listings",
            opts,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_q(q: &[(String, String)], k: &str) -> Option<String> {
        q.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.clone())
    }

    #[test]
    fn every_filter_emits_under_its_api_name() {
        let opts = ListAssistanceAwardsOptions::builder()
            .key("K1|K2")
            .fain("693JF72640002")
            .uri("URI1")
            .funding_opportunity_number("USDA-FS-2024-CWDG-CWSF")
            .uei("EKPQAJKKV549")
            .recipient("crisfield")
            .awarding_agency("6938")
            .funding_agency("069")
            .cfda_number("93.778|93.767")
            .assistance_type("F001")
            .record_type("2")
            .business_funds_indicator("NON")
            .disaster_emergency_fund_code("L")
            .award_date("2026-10-19")
            .award_date_gte("2026-01-01")
            .award_date_lte("2026-12-31")
            .fiscal_year("2027")
            .fiscal_year_gte("2025")
            .fiscal_year_lte("2027")
            .pop_start_date_gte("2026-02-01")
            .pop_start_date_lte("2026-02-28")
            .pop_end_date_gte("2029-01-01")
            .pop_end_date_lte("2029-12-31")
            .pop_state("AL|MD")
            .pop_country("USA")
            .pop_zip("36633")
            .obligated_gte("1000")
            .obligated_lte("5000000.5")
            .search("port infrastructure")
            .ordering("-pragmatic_obligations")
            .build();
        let q = opts.to_query();
        let want = [
            ("key", "K1|K2"),
            ("fain", "693JF72640002"),
            ("uri", "URI1"),
            ("funding_opportunity_number", "USDA-FS-2024-CWDG-CWSF"),
            ("uei", "EKPQAJKKV549"),
            ("recipient", "crisfield"),
            ("awarding_agency", "6938"),
            ("funding_agency", "069"),
            ("cfda_number", "93.778|93.767"),
            ("assistance_type", "F001"),
            ("record_type", "2"),
            ("business_funds_indicator", "NON"),
            ("disaster_emergency_fund_code", "L"),
            ("award_date", "2026-10-19"),
            ("award_date_gte", "2026-01-01"),
            ("award_date_lte", "2026-12-31"),
            ("fiscal_year", "2027"),
            ("fiscal_year_gte", "2025"),
            ("fiscal_year_lte", "2027"),
            ("pop_start_date_gte", "2026-02-01"),
            ("pop_start_date_lte", "2026-02-28"),
            ("pop_end_date_gte", "2029-01-01"),
            ("pop_end_date_lte", "2029-12-31"),
            ("pop_state", "AL|MD"),
            ("pop_country", "USA"),
            ("pop_zip", "36633"),
            ("obligated_gte", "1000"),
            ("obligated_lte", "5000000.5"),
            ("search", "port infrastructure"),
            ("ordering", "-pragmatic_obligations"),
        ];
        assert_eq!(q.len(), want.len(), "unexpected query {q:?}");
        for (k, v) in want {
            assert_eq!(get_q(&q, k).as_deref(), Some(v), "param {k}");
        }
    }

    #[test]
    fn empty_options_send_nothing() {
        let q = ListAssistanceAwardsOptions::builder().build().to_query();
        assert!(q.is_empty(), "expected empty query, got {q:?}");
    }

    #[test]
    fn list_sends_cursor_limit_and_shape() {
        let q = ListAssistanceAwardsOptions::builder()
            .limit(50u32)
            .cursor("WyIyMDI2LTEwLTE2Il0".to_string())
            .shape(crate::SHAPE_ASSISTANCE_AWARDS_MINIMAL)
            .build()
            .to_query();
        assert_eq!(get_q(&q, "limit").as_deref(), Some("50"));
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("WyIyMDI2LTEwLTE2Il0"));
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_ASSISTANCE_AWARDS_MINIMAL)
        );
    }

    #[test]
    fn joiner_only_when_flat() {
        let q = ListAssistanceAwardsOptions::builder()
            .joiner("__")
            .build()
            .to_query();
        assert!(!q.iter().any(|(k, _)| k == "joiner"));
        let q = ListAssistanceAwardsOptions::builder()
            .shape("key,recipient(uei)")
            .flat(true)
            .flat_lists(true)
            .joiner("__")
            .build()
            .to_query();
        assert_eq!(get_q(&q, "flat").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "flat_lists").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "joiner").as_deref(), Some("__"));
    }

    #[test]
    fn extra_passes_through() {
        let mut extra = BTreeMap::new();
        extra.insert("exact".to_string(), "true".to_string());
        let q = ListAssistanceAwardsOptions::builder()
            .extra(extra)
            .build()
            .to_query();
        assert_eq!(q, vec![("exact".to_string(), "true".to_string())]);
    }

    #[test]
    fn get_options_emit_shape_and_flattening() {
        let q = GetAssistanceAwardOptions::builder()
            .shape("key,listings(*)")
            .flat(true)
            .joiner("__")
            .build()
            .to_query();
        assert_eq!(get_q(&q, "shape").as_deref(), Some("key,listings(*)"));
        assert_eq!(get_q(&q, "flat").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "joiner").as_deref(), Some("__"));
        assert!(GetAssistanceAwardOptions::builder()
            .build()
            .to_query()
            .is_empty());
    }

    #[test]
    fn subresource_options_send_only_page_and_limit() {
        let q = AssistanceSubresourceOptions::builder()
            .page(2u32)
            .limit(500u32)
            .build()
            .to_query();
        assert_eq!(
            q,
            vec![
                ("page".to_string(), "2".to_string()),
                ("limit".to_string(), "500".to_string())
            ]
        );
    }

    #[tokio::test]
    async fn empty_key_is_a_validation_error_naming_the_method() {
        let client = Client::builder().api_key("x").build().expect("client");
        let errors = [
            client.get_assistance_grant("", None).await.map(|_| ()),
            client.get_assistance_loan("", None).await.map(|_| ()),
            client
                .get_assistance_direct_payment("", None)
                .await
                .map(|_| ()),
            client.get_assistance_other("", None).await.map(|_| ()),
            client
                .list_assistance_grant_transactions("", None)
                .await
                .map(|_| ()),
            client
                .list_assistance_loan_listings("", None)
                .await
                .map(|_| ()),
            client
                .list_assistance_direct_payment_transactions("", None)
                .await
                .map(|_| ()),
            client
                .list_assistance_other_listings("", None)
                .await
                .map(|_| ()),
        ];
        let methods = [
            "get_assistance_grant",
            "get_assistance_loan",
            "get_assistance_direct_payment",
            "get_assistance_other",
            "list_assistance_grant_transactions",
            "list_assistance_loan_listings",
            "list_assistance_direct_payment_transactions",
            "list_assistance_other_listings",
        ];
        for (result, method) in errors.into_iter().zip(methods) {
            match result {
                Err(Error::Validation { message, .. }) => {
                    assert_eq!(message, format!("{method}: key is required"));
                }
                other => panic!("{method}: expected Validation, got {other:?}"),
            }
        }
    }
}
