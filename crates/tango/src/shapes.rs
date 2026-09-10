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

/// Suggested list shape for
/// [`Client::list_sled_opportunities`](crate::Client::list_sled_opportunities).
///
/// `description` is detail-only on the API — its median is around 550
/// characters and its tail runs past 120,000 — so it is deliberately absent
/// here. Name it explicitly, or pass `verbose=true` via `extra`.
pub const SHAPE_SLED_OPPORTUNITIES_MINIMAL: &str = concat!(
    "opportunity_id,solicitation_number,solicitation_type,title,state,",
    "jurisdiction,agency,status,status_reason,posted_date,response_deadline,",
    "source_url,has_documents,first_seen_at,last_change_seen_at",
);

/// Suggested detail shape for
/// [`Client::get_sled_opportunity`](crate::Client::get_sled_opportunity).
///
/// Deliberately does not name `attachments(extracted_text)`. The document body
/// needs a Small plan and the API resolves it only for a caller who names the
/// leaf, so a default shape carrying it would make every detail fetch pay for a
/// document nobody asked to read. Ask for it explicitly instead:
///
/// ```text
/// opportunity_id,attachments(name,size_bytes,extracted_text)
/// ```
pub const SHAPE_SLED_OPPORTUNITIES_COMPREHENSIVE: &str = concat!(
    "opportunity_id,solicitation_number,solicitation_type,",
    "solicitation_type_source,title,description,state,jurisdiction,agency,",
    "status,status_reason,status_computed_at,source_status,source_url,",
    "posted_date,response_deadline,response_deadline_original,",
    "bid_opening_date,bid_opening_raw,category_codes,has_documents,",
    "first_seen_at,last_seen_at,last_change_seen_at,",
    "organization(*),contact(*),meta(*),attachments(*),revisions(*)",
);

/// Suggested shape for
/// [`Client::list_sled_opportunity_revisions`](crate::Client::list_sled_opportunity_revisions).
///
/// `changes` — the per-field before and after — is omitted on purpose: it needs
/// a Small plan, so naming it in a default shape would 403 a Free caller on a
/// field they never asked to gate. `changed_fields` names what moved at every
/// plan.
pub const SHAPE_SLED_REVISIONS_MINIMAL: &str =
    "observed_at,sequence,kind,changed_fields,source_declared";

/// Suggested list shape for
/// [`Client::list_sled_forecasts`](crate::Client::list_sled_forecasts).
pub const SHAPE_SLED_FORECASTS_MINIMAL: &str = concat!(
    "forecast_id,state,agency,title,estimated_advertisement_date,",
    "estimated_advertisement_raw,procurement_category,procurement_method,",
    "contract_number,incumbent_name,source_url,estimated_value(*)",
);

/// Suggested detail shape for
/// [`Client::get_sled_forecast`](crate::Client::get_sled_forecast).
pub const SHAPE_SLED_FORECASTS_COMPREHENSIVE: &str = concat!(
    "forecast_id,state,agency,title,description,",
    "estimated_advertisement_date,estimated_advertisement_raw,",
    "procurement_category,procurement_method,contract_term,contract_number,",
    "incumbent_name,mbe_dbe_goal,delivery_location,source_url,source_status,",
    "has_documents,first_seen_at,last_seen_at,",
    "organization(*),contact(*),estimated_value(*)",
);
