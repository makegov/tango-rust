//! `GET /api/vehicles/` and `GET /api/vehicles/{uuid}/` — list, stream,
//! and fetch federal contracting vehicles (GWACs, BPAs, IDIQs, etc.).

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt, push_opt_u32};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_vehicles`] and [`Client::iterate_vehicles`].
///
/// Mirrors `ListVehiclesOptions` in the Go SDK. Server enforces a strict
/// ordering allowlist; passing an unrecognised value will 400.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListVehiclesOptions {
    // ----- Pagination + shape -----
    /// 1-based page number. Mutually exclusive with [`cursor`](Self::cursor).
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100 on most endpoints).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor for cursor-paginated endpoints.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use a [`SHAPE_*`](crate::shapes)
    /// constant or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    // ----- Resource filters -----
    /// Joiner used between flattened keys when `flat=true`. Defaults to `.` server-side.
    #[builder(into)]
    pub joiner: Option<String>,
    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// Vehicle type filter (e.g. `"IDC"`).
    #[builder(into)]
    pub vehicle_type: Option<String>,
    /// Type of IDC filter (e.g. `"GWAC"`).
    #[builder(into)]
    pub type_of_idc: Option<String>,
    /// Contract type filter (e.g. `"FFP"`).
    #[builder(into)]
    pub contract_type: Option<String>,
    /// Set-aside filter (e.g. `"8A"`).
    #[builder(into)]
    pub set_aside: Option<String>,
    /// Who-can-use filter (e.g. `"All"`).
    #[builder(into)]
    pub who_can_use: Option<String>,
    /// NAICS code filter.
    #[builder(into)]
    pub naics_code: Option<String>,
    /// PSC code filter.
    #[builder(into)]
    pub psc_code: Option<String>,
    /// Program acronym filter (e.g. `"OASIS"`).
    #[builder(into)]
    pub program_acronym: Option<String>,
    /// Awarding agency CGAC code.
    #[builder(into)]
    pub agency: Option<String>,
    /// Organization ID filter.
    #[builder(into)]
    pub organization_id: Option<String>,
    /// Lower bound for total obligated dollars (inclusive).
    #[builder(into)]
    pub total_obligated_min: Option<String>,
    /// Upper bound for total obligated dollars (inclusive).
    #[builder(into)]
    pub total_obligated_max: Option<String>,
    /// Lower bound for the number of child IDVs.
    #[builder(into)]
    pub idv_count_min: Option<u32>,
    /// Upper bound for the number of child IDVs.
    #[builder(into)]
    pub idv_count_max: Option<u32>,
    /// Lower bound for the number of orders placed against the vehicle.
    #[builder(into)]
    pub order_count_min: Option<u32>,
    /// Upper bound for the number of orders placed against the vehicle.
    #[builder(into)]
    pub order_count_max: Option<u32>,
    /// `fiscal_year` filter.
    #[builder(into)]
    pub fiscal_year: Option<String>,
    /// Lower bound for award date (ISO `YYYY-MM-DD`).
    #[builder(into)]
    pub award_date_after: Option<String>,
    /// Upper bound for award date (ISO `YYYY-MM-DD`).
    #[builder(into)]
    pub award_date_before: Option<String>,
    /// Lower bound for last date to order.
    #[builder(into)]
    pub last_date_to_order_after: Option<String>,
    /// Upper bound for last date to order.
    #[builder(into)]
    pub last_date_to_order_before: Option<String>,
    /// Server-side sort spec. Server enforces a strict allowlist.
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListVehiclesOptions {
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
        push_opt(&mut q, "joiner", self.joiner.as_deref());
        push_opt(&mut q, "search", self.search.as_deref());
        push_opt(&mut q, "vehicle_type", self.vehicle_type.as_deref());
        push_opt(&mut q, "type_of_idc", self.type_of_idc.as_deref());
        push_opt(&mut q, "contract_type", self.contract_type.as_deref());
        push_opt(&mut q, "set_aside", self.set_aside.as_deref());
        push_opt(&mut q, "who_can_use", self.who_can_use.as_deref());
        push_opt(&mut q, "naics_code", self.naics_code.as_deref());
        push_opt(&mut q, "psc_code", self.psc_code.as_deref());
        push_opt(&mut q, "program_acronym", self.program_acronym.as_deref());
        push_opt(&mut q, "agency", self.agency.as_deref());
        push_opt(&mut q, "organization_id", self.organization_id.as_deref());
        push_opt(
            &mut q,
            "total_obligated_min",
            self.total_obligated_min.as_deref(),
        );
        push_opt(
            &mut q,
            "total_obligated_max",
            self.total_obligated_max.as_deref(),
        );
        push_opt_u32(&mut q, "idv_count_min", self.idv_count_min);
        push_opt_u32(&mut q, "idv_count_max", self.idv_count_max);
        push_opt_u32(&mut q, "order_count_min", self.order_count_min);
        push_opt_u32(&mut q, "order_count_max", self.order_count_max);
        push_opt(&mut q, "fiscal_year", self.fiscal_year.as_deref());
        push_opt(&mut q, "award_date_after", self.award_date_after.as_deref());
        push_opt(
            &mut q,
            "award_date_before",
            self.award_date_before.as_deref(),
        );
        push_opt(
            &mut q,
            "last_date_to_order_after",
            self.last_date_to_order_after.as_deref(),
        );
        push_opt(
            &mut q,
            "last_date_to_order_before",
            self.last_date_to_order_before.as_deref(),
        );
        push_opt(&mut q, "ordering", self.ordering.as_deref());

        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

/// Options for [`Client::get_vehicle`]. Mirrors the Go SDK's
/// `GetEntityOptions` for the vehicle detail endpoint.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetVehicleOptions {
    /// Comma-separated field selector for dynamic response shaping.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,
}

impl GetVehicleOptions {
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
    /// `GET /api/vehicles/` — one page of contracting-vehicle records.
    pub async fn list_vehicles(&self, opts: ListVehiclesOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/vehicles/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/vehicles/{uuid}/` — fetch a single contracting vehicle.
    pub async fn get_vehicle(&self, uuid: &str, opts: Option<GetVehicleOptions>) -> Result<Record> {
        if uuid.is_empty() {
            return Err(Error::Validation {
                message: "get_vehicle: uuid is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/vehicles/{}/", urlencoding(uuid));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every contracting-vehicle record matching `opts`.
    pub fn iterate_vehicles(&self, opts: ListVehiclesOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_vehicles(next).await })
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
    fn list_vehicles_emits_string_filters() {
        let opts = ListVehiclesOptions::builder()
            .search("oasis")
            .vehicle_type("IDC")
            .type_of_idc("GWAC")
            .set_aside("8A")
            .naics_code("541512")
            .psc_code("D302")
            .program_acronym("OASIS")
            .agency("9700")
            .fiscal_year("2024")
            .ordering("total_obligated")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "search").as_deref(), Some("oasis"));
        assert_eq!(get_q(&q, "vehicle_type").as_deref(), Some("IDC"));
        assert_eq!(get_q(&q, "type_of_idc").as_deref(), Some("GWAC"));
        assert_eq!(get_q(&q, "set_aside").as_deref(), Some("8A"));
        assert_eq!(get_q(&q, "naics_code").as_deref(), Some("541512"));
        assert_eq!(get_q(&q, "psc_code").as_deref(), Some("D302"));
        assert_eq!(get_q(&q, "program_acronym").as_deref(), Some("OASIS"));
        assert_eq!(get_q(&q, "agency").as_deref(), Some("9700"));
        assert_eq!(get_q(&q, "fiscal_year").as_deref(), Some("2024"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("total_obligated"));
    }

    #[test]
    fn list_vehicles_emits_int_counts() {
        let opts = ListVehiclesOptions::builder()
            .idv_count_min(3u32)
            .idv_count_max(50u32)
            .order_count_min(100u32)
            .order_count_max(9999u32)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "idv_count_min").as_deref(), Some("3"));
        assert_eq!(get_q(&q, "idv_count_max").as_deref(), Some("50"));
        assert_eq!(get_q(&q, "order_count_min").as_deref(), Some("100"));
        assert_eq!(get_q(&q, "order_count_max").as_deref(), Some("9999"));
    }

    #[test]
    fn list_vehicles_zero_counts_omitted() {
        let opts = ListVehiclesOptions::builder()
            .idv_count_min(0u32)
            .order_count_max(0u32)
            .build();
        let q = opts.to_query();
        assert!(!q.iter().any(|(k, _)| k == "idv_count_min"));
        assert!(!q.iter().any(|(k, _)| k == "order_count_max"));
    }

    #[test]
    fn list_vehicles_extra_map() {
        let mut extra = BTreeMap::new();
        extra.insert("version".to_string(), "v2".to_string());
        let opts = ListVehiclesOptions::builder().extra(extra).build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "version").as_deref(), Some("v2"));
    }

    #[test]
    fn get_vehicle_options_emits_shape_and_flat() {
        let opts = GetVehicleOptions::builder()
            .shape("vehicles(minimal)")
            .flat(true)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "shape").as_deref(), Some("vehicles(minimal)"));
        assert_eq!(get_q(&q, "flat").as_deref(), Some("true"));
        assert!(!q.iter().any(|(k, _)| k == "flat_lists"));
    }

    #[tokio::test]
    async fn get_vehicle_empty_uuid_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client.get_vehicle("", None).await.expect_err("must error");
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("uuid"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
