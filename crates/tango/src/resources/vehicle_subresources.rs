//! `GET /api/vehicles/{uuid}/awardees/` and
//! `GET /api/vehicles/{uuid}/orders/` — list and stream the awardees
//! (child-IDV-holding entities) and task orders for a contracting vehicle.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::apply_pagination;
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::sync::Arc;

/// Options for [`Client::list_vehicle_awardees`] and
/// [`Client::iterate_vehicle_awardees`]. The endpoint only accepts the
/// shared pagination + shaping fields.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListVehicleAwardeesOptions {
    /// 1-based page number. Mutually exclusive with [`cursor`](Self::cursor).
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
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
}

impl ListVehicleAwardeesOptions {
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
        q
    }
}

/// Options for [`Client::list_vehicle_orders`] and
/// [`Client::iterate_vehicle_orders`]. The endpoint only accepts the
/// shared pagination + shaping fields.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListVehicleOrdersOptions {
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
}

impl ListVehicleOrdersOptions {
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
        q
    }
}

impl Client {
    /// `GET /api/vehicles/{uuid}/awardees/` — entities holding child IDVs
    /// under this contracting vehicle.
    pub async fn list_vehicle_awardees(
        &self,
        uuid: &str,
        opts: ListVehicleAwardeesOptions,
    ) -> Result<Page<Record>> {
        list_vehicle_awardees_inner(self, uuid, opts).await
    }

    /// `GET /api/vehicles/{uuid}/orders/` — task orders placed under this
    /// vehicle's child IDVs.
    pub async fn list_vehicle_orders(
        &self,
        uuid: &str,
        opts: ListVehicleOrdersOptions,
    ) -> Result<Page<Record>> {
        list_vehicle_orders_inner(self, uuid, opts).await
    }

    /// Stream every awardee under this contracting vehicle.
    pub fn iterate_vehicle_awardees(
        &self,
        uuid: &str,
        opts: ListVehicleAwardeesOptions,
    ) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let uuid = Arc::new(uuid.to_string());
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            let uuid = uuid.clone();
            Box::pin(async move { list_vehicle_awardees_inner(&client, &uuid, next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// Stream every task order placed under this contracting vehicle.
    pub fn iterate_vehicle_orders(
        &self,
        uuid: &str,
        opts: ListVehicleOrdersOptions,
    ) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let uuid = Arc::new(uuid.to_string());
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            let uuid = uuid.clone();
            Box::pin(async move { list_vehicle_orders_inner(&client, &uuid, next).await })
        });
        PageStream::new(self.clone(), fetch)
    }
}

async fn list_vehicle_awardees_inner(
    client: &Client,
    uuid: &str,
    opts: ListVehicleAwardeesOptions,
) -> Result<Page<Record>> {
    if uuid.is_empty() {
        return Err(Error::Validation {
            message: "list_vehicle_awardees: uuid is required".into(),
            response: None,
        });
    }
    let q = opts.to_query();
    let path = format!("/api/vehicles/{}/awardees/", urlencoding(uuid));
    let bytes = client.get_bytes(&path, &q).await?;
    Page::decode(&bytes)
}

async fn list_vehicle_orders_inner(
    client: &Client,
    uuid: &str,
    opts: ListVehicleOrdersOptions,
) -> Result<Page<Record>> {
    if uuid.is_empty() {
        return Err(Error::Validation {
            message: "list_vehicle_orders: uuid is required".into(),
            response: None,
        });
    }
    let q = opts.to_query();
    let path = format!("/api/vehicles/{}/orders/", urlencoding(uuid));
    let bytes = client.get_bytes(&path, &q).await?;
    Page::decode(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn awardees_opts_emit_pagination() {
        let opts = ListVehicleAwardeesOptions::builder()
            .page(2u32)
            .limit(25u32)
            .shape("vehicle_awardees(minimal)")
            .build();
        let q = opts.to_query();
        assert!(q.contains(&("page".into(), "2".into())));
        assert!(q.contains(&("limit".into(), "25".into())));
        assert!(q.contains(&("shape".into(), "vehicle_awardees(minimal)".into())));
    }

    #[test]
    fn orders_opts_emit_flat() {
        let opts = ListVehicleOrdersOptions::builder()
            .flat(true)
            .flat_lists(true)
            .build();
        let q = opts.to_query();
        assert!(q.contains(&("flat".into(), "true".into())));
        assert!(q.contains(&("flat_lists".into(), "true".into())));
    }

    #[tokio::test]
    async fn list_vehicle_awardees_empty_uuid_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .list_vehicle_awardees("", ListVehicleAwardeesOptions::default())
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("uuid"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn list_vehicle_orders_empty_uuid_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .list_vehicle_orders("", ListVehicleOrdersOptions::default())
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("uuid"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
