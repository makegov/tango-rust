//! Shape presets for the Tango API's dynamic response-shaping feature.
//!
//! Every list/get endpoint accepts a `shape=` query parameter selecting which
//! fields the API returns. Use one of the [`shapes`](self) constants as a
//! sensible default, or pass a custom comma-separated field list of your own.
//!
//! These constants mirror the canonical defaults in `tango-node` and
//! `tango-python` so identical calls return identical bodies across SDKs.

/// The public Tango SaaS endpoint.
pub const DEFAULT_BASE_URL: &str = "https://tango.makegov.com";

/// Default shape for [`Client::list_contracts`](crate::Client::list_contracts).
pub const SHAPE_CONTRACTS_MINIMAL: &str =
    "key,piid,award_date,recipient(display_name),description,total_contract_value";

/// Default shape for [`Client::list_entities`](crate::Client::list_entities).
pub const SHAPE_ENTITIES_MINIMAL: &str = "uei,legal_business_name,cage_code,business_types";

/// Default shape for [`Client::get_entity`](crate::Client::get_entity).
pub const SHAPE_ENTITIES_COMPREHENSIVE: &str = concat!(
    "uei,legal_business_name,dba_name,cage_code,",
    "business_types,primary_naics,naics_codes,psc_codes,",
    "email_address,entity_url,description,capabilities,keywords,",
    "physical_address,mailing_address,",
    "federal_obligations(*),congressional_district",
);

/// Default shape for [`Client::list_forecasts`](crate::Client::list_forecasts).
pub const SHAPE_FORECASTS_MINIMAL: &str =
    "id,title,anticipated_award_date,fiscal_year,naics_code,status";

/// Default shape for [`Client::list_opportunities`](crate::Client::list_opportunities).
pub const SHAPE_OPPORTUNITIES_MINIMAL: &str =
    "opportunity_id,title,solicitation_number,response_deadline,active";

/// Default shape for [`Client::list_notices`](crate::Client::list_notices).
pub const SHAPE_NOTICES_MINIMAL: &str = "notice_id,title,solicitation_number,posted_date";

/// Default shape for [`Client::list_protests`](crate::Client::list_protests).
pub const SHAPE_PROTESTS_MINIMAL: &str =
    "case_id,case_number,title,source_system,outcome,filed_date";

/// Default shape for [`Client::list_grants`](crate::Client::list_grants).
pub const SHAPE_GRANTS_MINIMAL: &str = "grant_id,opportunity_number,title,status(*),agency_code";

/// Default shape for [`Client::list_idvs`](crate::Client::list_idvs).
pub const SHAPE_IDVS_MINIMAL: &str = concat!(
    "key,piid,award_date,recipient(display_name,uei),description,",
    "total_contract_value,obligated,idv_type",
);

/// Default shape for [`Client::get_idv`](crate::Client::get_idv).
pub const SHAPE_IDVS_COMPREHENSIVE: &str = concat!(
    "key,piid,award_date,description,fiscal_year,total_contract_value,obligated,",
    "idv_type,multiple_or_single_award_idv,type_of_idc,period_of_performance(start_date,last_date_to_order),",
    "recipient(display_name,legal_business_name,uei,cage),",
    "awarding_office(*),funding_office(*),place_of_performance(*),parent_award(key,piid),",
    "competition(*),legislative_mandates(*),transactions(*),subawards_summary(*)",
);

/// Default shape for [`Client::list_vehicles`](crate::Client::list_vehicles).
pub const SHAPE_VEHICLES_MINIMAL: &str = concat!(
    "uuid,solicitation_identifier,is_synthetic_solicitation,program_acronym,",
    "organization_id,organization,vehicle_type,description,",
    "idv_count,awardee_count,order_count,total_obligated,",
    "vehicle_obligations,vehicle_contracts_value,latest_award_date,",
    "solicitation_title,solicitation_date",
);

/// Default shape for [`Client::get_vehicle`](crate::Client::get_vehicle).
pub const SHAPE_VEHICLES_COMPREHENSIVE: &str = concat!(
    "uuid,solicitation_identifier,is_synthetic_solicitation,agency_id,program_acronym,",
    "organization_id,organization(*),vehicle_type,who_can_use,",
    "solicitation_title,solicitation_description,solicitation_date,opportunity_id,",
    "naics_code,psc_code,set_aside,",
    "fiscal_year,award_date,latest_award_date,last_date_to_order,",
    "description,idv_count,awardee_count,order_count,total_obligated,",
    "vehicle_obligations,vehicle_contracts_value,",
    "type_of_idc,contract_type,metrics(*)",
);

/// Default shape for [`Client::list_vehicle_awardees`](crate::Client::list_vehicle_awardees).
pub const SHAPE_VEHICLE_AWARDEES_MINIMAL: &str = concat!(
    "uuid,key,piid,award_date,title,order_count,idv_obligations,",
    "idv_contracts_value,recipient(display_name,uei)",
);

/// Default shape for [`Client::list_vehicle_orders`](crate::Client::list_vehicle_orders).
pub const SHAPE_VEHICLE_ORDERS_MINIMAL: &str = concat!(
    "key,piid,award_date,obligated,total_contract_value,description,",
    "recipient(display_name,uei)",
);

/// Default shape for [`Client::list_organizations`](crate::Client::list_organizations).
pub const SHAPE_ORGANIZATIONS_MINIMAL: &str = "key,fh_key,name,level,type,short_name";

/// Default shape for [`Client::list_otas`](crate::Client::list_otas).
pub const SHAPE_OTAS_MINIMAL: &str = concat!(
    "key,piid,award_date,recipient(display_name,uei),description,",
    "total_contract_value,obligated",
);

/// Default shape for [`Client::list_otidvs`](crate::Client::list_otidvs).
pub const SHAPE_OTIDVS_MINIMAL: &str = concat!(
    "key,piid,award_date,recipient(display_name,uei),description,",
    "total_contract_value,obligated,idv_type",
);

/// Default shape for [`Client::list_subawards`](crate::Client::list_subawards).
///
/// The Tango API rejects `id` and `amount` in subaward shapes; stick to the
/// canonical fields below or roll your own subset.
pub const SHAPE_SUBAWARDS_MINIMAL: &str =
    "award_key,prime_recipient(uei,display_name),subaward_recipient(uei,display_name)";

/// Default shape for [`Client::list_gsa_elibrary_contracts`](crate::Client::list_gsa_elibrary_contracts).
pub const SHAPE_GSA_ELIBRARY_CONTRACTS_MINIMAL: &str = concat!(
    "uuid,contract_number,schedule,recipient(display_name,uei),",
    "idv(key,award_date)",
);

/// Default shape for [`Client::list_itdashboard`](crate::Client::list_itdashboard).
/// Mirrors the API's `INVESTMENT_LIST_DEFAULT_SHAPE`.
pub const SHAPE_ITDASHBOARD_INVESTMENTS_MINIMAL: &str = concat!(
    "uii,agency_name,bureau_name,investment_title,",
    "type_of_investment,part_of_it_portfolio,updated_time,url",
);

/// Default shape for [`Client::get_itdashboard`](crate::Client::get_itdashboard).
/// Mirrors the API's `INVESTMENT_RETRIEVE_DEFAULT_SHAPE`.
pub const SHAPE_ITDASHBOARD_INVESTMENTS_COMPREHENSIVE: &str = concat!(
    "uii,agency_code,agency_name,bureau_code,bureau_name,",
    "investment_title,type_of_investment,part_of_it_portfolio,",
    "updated_time,url",
);
