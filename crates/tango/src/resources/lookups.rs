//! Reference / lookup endpoints — organizations, NAICS, PSC, MAS SINs,
//! Assistance Listings (CFDA), business types, offices, departments.
//!
//! These are read-only catalogues exposed by the Tango API for cross-referencing
//! contracts, IDVs, and other records.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt, push_opt_bool};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

// =====================================================================
// Organizations
// =====================================================================

/// Options for [`Client::list_organizations`] / [`Client::iterate_organizations`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListOrganizationsOptions {
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

    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// Filter by organization type.
    #[builder(into)]
    pub r#type: Option<String>,
    /// Filter by hierarchy level (e.g. `"1"` for departments).
    #[builder(into)]
    pub level: Option<String>,
    /// CGAC code filter.
    #[builder(into)]
    pub cgac: Option<String>,
    /// Parent organization filter.
    #[builder(into)]
    pub parent: Option<String>,
    /// When `Some(true)`, include inactive organizations; `Some(false)`
    /// excludes them. `None` leaves it to the server default.
    pub include_inactive: Option<bool>,

    /// Escape hatch for filter keys not first-classed here.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListOrganizationsOptions {
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
        push_opt(&mut q, "search", self.search.as_deref());
        push_opt(&mut q, "type", self.r#type.as_deref());
        push_opt(&mut q, "level", self.level.as_deref());
        push_opt(&mut q, "cgac", self.cgac.as_deref());
        push_opt(&mut q, "parent", self.parent.as_deref());
        push_opt_bool(&mut q, "include_inactive", self.include_inactive);
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

// =====================================================================
// NAICS
// =====================================================================

/// Options for [`Client::list_naics`] / [`Client::iterate_naics`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListNaicsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
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

    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// SBA revenue size standard (exact match).
    #[builder(into)]
    pub revenue_limit: Option<String>,
    /// SBA employee size standard (exact match).
    #[builder(into)]
    pub employee_limit: Option<String>,
    /// Lower bound for SBA revenue size standard.
    #[builder(into)]
    pub revenue_limit_gte: Option<String>,
    /// Upper bound for SBA revenue size standard.
    #[builder(into)]
    pub revenue_limit_lte: Option<String>,
    /// Lower bound for SBA employee size standard.
    #[builder(into)]
    pub employee_limit_gte: Option<String>,
    /// Upper bound for SBA employee size standard.
    #[builder(into)]
    pub employee_limit_lte: Option<String>,

    /// Escape hatch for filter keys not first-classed here.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListNaicsOptions {
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
        push_opt(&mut q, "search", self.search.as_deref());
        push_opt(&mut q, "revenue_limit", self.revenue_limit.as_deref());
        push_opt(&mut q, "employee_limit", self.employee_limit.as_deref());
        push_opt(
            &mut q,
            "revenue_limit_gte",
            self.revenue_limit_gte.as_deref(),
        );
        push_opt(
            &mut q,
            "revenue_limit_lte",
            self.revenue_limit_lte.as_deref(),
        );
        push_opt(
            &mut q,
            "employee_limit_gte",
            self.employee_limit_gte.as_deref(),
        );
        push_opt(
            &mut q,
            "employee_limit_lte",
            self.employee_limit_lte.as_deref(),
        );
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

// =====================================================================
// PSC
// =====================================================================

/// Options for [`Client::list_psc`] / [`Client::iterate_psc`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListPscOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
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

    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,

    /// Escape hatch for filter keys not first-classed here.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListPscOptions {
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
        push_opt(&mut q, "search", self.search.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

// =====================================================================
// MAS SINs
// =====================================================================

/// Options for [`Client::list_mas_sins`] / [`Client::iterate_mas_sins`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListMasSinsOptions {
    /// 1-based page number.
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

    /// Free-text search filter (SIN number, title, description).
    #[builder(into)]
    pub search: Option<String>,

    /// Escape hatch for filter keys not first-classed here.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListMasSinsOptions {
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
        push_opt(&mut q, "search", self.search.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

// =====================================================================
// Assistance Listings (CFDA)
// =====================================================================

/// Options for [`Client::list_assistance_listings`] / [`Client::iterate_assistance_listings`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListAssistanceListingsOptions {
    /// 1-based page number.
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

    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,

    /// Escape hatch for filter keys not first-classed here.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListAssistanceListingsOptions {
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
        push_opt(&mut q, "search", self.search.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

// =====================================================================
// Business Types
// =====================================================================

/// Options for [`Client::list_business_types`] / [`Client::iterate_business_types`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListBusinessTypesOptions {
    /// 1-based page number.
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

    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,

    /// Escape hatch for filter keys not first-classed here.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListBusinessTypesOptions {
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
        push_opt(&mut q, "search", self.search.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

// =====================================================================
// Offices
// =====================================================================

/// Options for [`Client::list_offices`] / [`Client::iterate_offices`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListOfficesOptions {
    /// 1-based page number.
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

    /// Free-text search filter (office name, code).
    #[builder(into)]
    pub search: Option<String>,

    /// Escape hatch for filter keys not first-classed here.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListOfficesOptions {
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
        push_opt(&mut q, "search", self.search.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

// =====================================================================
// Departments (deprecated)
// =====================================================================

/// Options for [`Client::list_departments`] / [`Client::iterate_departments`].
///
/// Deprecated in spirit: prefer [`ListOrganizationsOptions`] with `level = "1"`.
/// The standalone `/api/departments/` endpoint is retained for backward
/// compatibility.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListDepartmentsOptions {
    /// 1-based page number.
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

    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,

    /// Escape hatch for filter keys not first-classed here.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListDepartmentsOptions {
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
        push_opt(&mut q, "search", self.search.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

// =====================================================================
// Client methods
// =====================================================================

impl Client {
    // ----- Organizations -----

    /// `GET /api/organizations/` — one page of organization records.
    pub async fn list_organizations(&self, opts: ListOrganizationsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/organizations/", &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every organization record matching `opts`.
    pub fn iterate_organizations(&self, opts: ListOrganizationsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_organizations(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/organizations/{key}/` — fetch a single organization.
    pub async fn get_organization(&self, key: &str) -> Result<Record> {
        if key.is_empty() {
            return Err(Error::Validation {
                message: "get_organization: organization key is required".into(),
                response: None,
            });
        }
        let path = format!("/api/organizations/{}/", urlencoding(key));
        self.get_json::<Record>(&path, &[]).await
    }

    // ----- NAICS -----

    /// `GET /api/naics/` — one page of NAICS records.
    pub async fn list_naics(&self, opts: ListNaicsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/naics/", &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every NAICS record matching `opts`.
    pub fn iterate_naics(&self, opts: ListNaicsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_naics(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/naics/{code}/` — fetch a single NAICS record.
    pub async fn get_naics(&self, code: &str) -> Result<Record> {
        if code.is_empty() {
            return Err(Error::Validation {
                message: "get_naics: NAICS code is required".into(),
                response: None,
            });
        }
        let path = format!("/api/naics/{}/", urlencoding(code));
        self.get_json::<Record>(&path, &[]).await
    }

    // ----- PSC -----

    /// `GET /api/psc/` — one page of PSC (Product/Service Code) records.
    pub async fn list_psc(&self, opts: ListPscOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/psc/", &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every PSC record matching `opts`.
    pub fn iterate_psc(&self, opts: ListPscOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_psc(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/psc/{code}/` — fetch a single PSC record.
    pub async fn get_psc(&self, code: &str) -> Result<Record> {
        if code.is_empty() {
            return Err(Error::Validation {
                message: "get_psc: PSC code is required".into(),
                response: None,
            });
        }
        let path = format!("/api/psc/{}/", urlencoding(code));
        self.get_json::<Record>(&path, &[]).await
    }

    // ----- MAS SINs -----

    /// `GET /api/mas_sins/` — one page of GSA MAS Special Item Number records.
    pub async fn list_mas_sins(&self, opts: ListMasSinsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/mas_sins/", &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every MAS SIN record matching `opts`.
    pub fn iterate_mas_sins(&self, opts: ListMasSinsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_mas_sins(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/mas_sins/{sin}/` — fetch a single MAS SIN record.
    pub async fn get_mas_sin(&self, sin: &str) -> Result<Record> {
        if sin.is_empty() {
            return Err(Error::Validation {
                message: "get_mas_sin: MAS SIN is required".into(),
                response: None,
            });
        }
        let path = format!("/api/mas_sins/{}/", urlencoding(sin));
        self.get_json::<Record>(&path, &[]).await
    }

    // ----- Assistance Listings -----

    /// `GET /api/assistance_listings/` — one page of CFDA program records.
    pub async fn list_assistance_listings(
        &self,
        opts: ListAssistanceListingsOptions,
    ) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/assistance_listings/", &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every Assistance Listing record matching `opts`.
    pub fn iterate_assistance_listings(
        &self,
        opts: ListAssistanceListingsOptions,
    ) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_assistance_listings(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/assistance_listings/{number}/` — fetch a single Assistance
    /// Listing (CFDA program) by its CFDA number (e.g. `"10.001"`).
    pub async fn get_assistance_listing(&self, number: &str) -> Result<Record> {
        if number.is_empty() {
            return Err(Error::Validation {
                message: "get_assistance_listing: assistance listing number is required".into(),
                response: None,
            });
        }
        let path = format!("/api/assistance_listings/{}/", urlencoding(number));
        self.get_json::<Record>(&path, &[]).await
    }

    // ----- Business Types -----

    /// `GET /api/business_types/` — one page of business-type reference
    /// records (SBA / SAM.gov socioeconomic designations).
    pub async fn list_business_types(
        &self,
        opts: ListBusinessTypesOptions,
    ) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/business_types/", &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every business-type record matching `opts`.
    pub fn iterate_business_types(&self, opts: ListBusinessTypesOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_business_types(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/business_types/{code}/` — fetch a single business-type
    /// reference record by its short code.
    pub async fn get_business_type(&self, code: &str) -> Result<Record> {
        if code.is_empty() {
            return Err(Error::Validation {
                message: "get_business_type: business type code is required".into(),
                response: None,
            });
        }
        let path = format!("/api/business_types/{}/", urlencoding(code));
        self.get_json::<Record>(&path, &[]).await
    }

    // ----- Offices -----

    /// `GET /api/offices/` — one page of federal contracting office records.
    pub async fn list_offices(&self, opts: ListOfficesOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/offices/", &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every office record matching `opts`.
    pub fn iterate_offices(&self, opts: ListOfficesOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_offices(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/offices/{code}/` — fetch a single office record by its
    /// FPDS-NG office code.
    pub async fn get_office(&self, code: &str) -> Result<Record> {
        if code.is_empty() {
            return Err(Error::Validation {
                message: "get_office: office code is required".into(),
                response: None,
            });
        }
        let path = format!("/api/offices/{}/", urlencoding(code));
        self.get_json::<Record>(&path, &[]).await
    }

    // ----- Departments -----

    /// `GET /api/departments/` — one page of department records.
    ///
    /// Prefer [`Client::list_organizations`] with `level = "1"` for new code.
    /// The standalone `/api/departments/` endpoint is retained for backward
    /// compatibility.
    pub async fn list_departments(&self, opts: ListDepartmentsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/departments/", &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every department record matching `opts`.
    pub fn iterate_departments(&self, opts: ListDepartmentsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_departments(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/departments/{code}/` — fetch a single department by code
    /// (typically the CGAC department code, e.g. `"097"` for DoD).
    pub async fn get_department(&self, code: &str) -> Result<Record> {
        if code.is_empty() {
            return Err(Error::Validation {
                message: "get_department: department code is required".into(),
                response: None,
            });
        }
        let path = format!("/api/departments/{}/", urlencoding(code));
        self.get_json::<Record>(&path, &[]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_q(q: &[(String, String)], k: &str) -> Option<String> {
        q.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.clone())
    }

    // ----- Organizations -----

    #[test]
    fn organizations_emits_filters() {
        let opts = ListOrganizationsOptions::builder()
            .search("Defense")
            .r#type("agency")
            .level("1")
            .cgac("097")
            .parent("DOD")
            .include_inactive(false)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "search").as_deref(), Some("Defense"));
        assert_eq!(get_q(&q, "type").as_deref(), Some("agency"));
        assert_eq!(get_q(&q, "level").as_deref(), Some("1"));
        assert_eq!(get_q(&q, "cgac").as_deref(), Some("097"));
        assert_eq!(get_q(&q, "parent").as_deref(), Some("DOD"));
        assert_eq!(get_q(&q, "include_inactive").as_deref(), Some("false"));
    }

    #[tokio::test]
    async fn get_organization_empty_key_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let err = c.get_organization("").await.expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    // ----- NAICS -----

    #[test]
    fn naics_emits_size_filters() {
        let opts = ListNaicsOptions::builder()
            .search("software")
            .revenue_limit_gte("1000000")
            .employee_limit_lte("500")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "search").as_deref(), Some("software"));
        assert_eq!(get_q(&q, "revenue_limit_gte").as_deref(), Some("1000000"));
        assert_eq!(get_q(&q, "employee_limit_lte").as_deref(), Some("500"));
    }

    #[tokio::test]
    async fn get_naics_empty_code_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let err = c.get_naics("").await.expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    // ----- PSC -----

    #[test]
    fn psc_emits_search() {
        let opts = ListPscOptions::builder().search("services").build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "search").as_deref(), Some("services"));
    }

    #[tokio::test]
    async fn get_psc_empty_code_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let err = c.get_psc("").await.expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    // ----- MAS SINs -----

    #[test]
    fn mas_sins_emits_search() {
        let opts = ListMasSinsOptions::builder().search("54151S").build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "search").as_deref(), Some("54151S"));
    }

    #[tokio::test]
    async fn get_mas_sin_empty_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let err = c.get_mas_sin("").await.expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    // ----- Assistance Listings -----

    #[test]
    fn assistance_listings_paginates() {
        let opts = ListAssistanceListingsOptions::builder()
            .page(2u32)
            .limit(50u32)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "page").as_deref(), Some("2"));
        assert_eq!(get_q(&q, "limit").as_deref(), Some("50"));
    }

    #[tokio::test]
    async fn get_assistance_listing_empty_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let err = c.get_assistance_listing("").await.expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    // ----- Business Types -----

    #[test]
    fn business_types_passes_shape() {
        let opts = ListBusinessTypesOptions::builder()
            .shape("code,name")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "shape").as_deref(), Some("code,name"));
    }

    #[tokio::test]
    async fn get_business_type_empty_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let err = c.get_business_type("").await.expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    // ----- Offices -----

    #[test]
    fn offices_emits_search() {
        let opts = ListOfficesOptions::builder().search("FA8650").build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "search").as_deref(), Some("FA8650"));
    }

    #[tokio::test]
    async fn get_office_empty_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let err = c.get_office("").await.expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    // ----- Departments -----

    #[test]
    fn departments_passes_cursor() {
        let opts = ListDepartmentsOptions::builder()
            .cursor("abc".to_string())
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("abc"));
    }

    #[tokio::test]
    async fn get_department_empty_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let err = c.get_department("").await.expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    // ----- Extra escape hatch -----

    #[test]
    fn extra_keys_pass_through() {
        let mut extra = BTreeMap::new();
        extra.insert("custom".into(), "value".into());
        let opts = ListNaicsOptions::builder().extra(extra).build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "custom").as_deref(), Some("value"));
    }

    #[test]
    fn empty_extra_value_is_skipped() {
        let mut extra = BTreeMap::new();
        extra.insert("skip".into(), String::new());
        let opts = ListPscOptions::builder().extra(extra).build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "skip"), None);
    }
}
