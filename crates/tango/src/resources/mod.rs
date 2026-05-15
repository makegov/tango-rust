//! Resource endpoints. Each submodule binds one resource family to a set of
//! methods on [`Client`](crate::Client) and exports its `*Options` builders.

pub(crate) mod agencies;
pub(crate) mod contracts;
pub(crate) mod entities;
pub(crate) mod entity_subresources;
pub(crate) mod gsa;
pub(crate) mod idv_subresources;
pub(crate) mod idvs;
pub(crate) mod itdashboard;
pub(crate) mod lcats;
pub(crate) mod lookups;
pub(crate) mod meta;
pub(crate) mod metrics;
pub(crate) mod opportunities;
pub(crate) mod otas;
pub(crate) mod protests;
pub(crate) mod resolve_validate;
pub(crate) mod subawards;
pub(crate) mod vehicle_subresources;
pub(crate) mod vehicles;
pub(crate) mod webhooks_api;

pub use agencies::{
    AgencyContractsOptions, GetAgencyOptions, ListAgenciesOptions,
    ListAgencyAwardingContractsOptions, ListAgencyFundingContractsOptions,
};
pub use contracts::ListContractsOptions;
pub use entities::{GetEntityOptions, ListEntitiesOptions};
pub use entity_subresources::EntitySubresourceOptions;
pub use gsa::{GetGsaElibraryContractOptions, ListGsaElibraryContractsOptions};
pub use idv_subresources::IdvSubresourceOptions;
pub use idvs::{GetIDVOptions, ListIDVsOptions};
pub use itdashboard::{GetItdashboardOptions, ListItdashboardOptions};
pub use lcats::ListLcatsOptions;
pub use lookups::{
    ListAssistanceListingsOptions, ListBusinessTypesOptions, ListDepartmentsOptions,
    ListMasSinsOptions, ListNaicsOptions, ListOfficesOptions, ListOrganizationsOptions,
    ListPscOptions,
};
pub use meta::ListApiKeysOptions;
pub use metrics::{
    ListMetricsOptions, METRICS_OWNER_ENTITY, METRICS_OWNER_NAICS, METRICS_OWNER_PSC,
};
pub use opportunities::{
    ListForecastsOptions, ListGrantsOptions, ListNoticesOptions, ListOpportunitiesOptions,
    SearchOpportunityAttachmentsOptions,
};
pub use otas::{
    GetOTAOptions, GetOTIDVOptions, ListOTAsOptions, ListOTIDVAwardsOptions, ListOTIDVsOptions,
};
pub use protests::{GetProtestOptions, ListProtestsOptions};
pub use subawards::ListSubawardsOptions;
pub use vehicle_subresources::{ListVehicleAwardeesOptions, ListVehicleOrdersOptions};
pub use vehicles::{GetVehicleOptions, ListVehiclesOptions};
// Other resources re-export their option types from this module once their
// Phase 2 agent fills them in.

// Wave E (webhooks_api + resolve_validate): no new option types. Webhook list
// endpoints reuse `crate::ListOptions`; resolve/validate take typed inputs
// from `crate::models`.
