//! Typed records for the financial-assistance award endpoints under `/api/assistance/` (Tango API 5.12.0).
//!
//! [`AssistanceAwardRecord`] carries the fields of the API's default award shape, [`AssistanceTransaction`] one row of an award's `/transactions/` sub-route and [`AssistanceAwardListing`] one row of its `/listings/` sub-route.

use crate::Record;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A financial-assistance award: a grant, loan, direct payment or other assistance award from USAspending.
///
/// Returned by [`Client::get_assistance_grant`](crate::Client::get_assistance_grant), [`get_assistance_loan`](crate::Client::get_assistance_loan), [`get_assistance_direct_payment`](crate::Client::get_assistance_direct_payment) and [`get_assistance_other`](crate::Client::get_assistance_other), and decoded from a list row with [`AssistanceAwardRecord::from_record`].
/// Its fields are those of the API's default shape, [`SHAPE_ASSISTANCE_AWARDS_MINIMAL`](crate::SHAPE_ASSISTANCE_AWARDS_MINIMAL).
///
/// Every field is optional because the response follows the requested `shape`: a field the caller did not ask for is ABSENT rather than null.
/// Fields outside the default shape (`summary`, `former_award_keys`, `listings`, `transactions`, …) land in [`extra`](Self::extra) when named.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AssistanceAwardRecord {
    /// Tango's award key, and the path segment every detail and sub-route method takes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Federal Award Identification Number, carried by non-aggregate records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fain: Option<String>,

    /// Unique Record Identifier, carried by aggregate records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    /// ISO date of the award.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub award_date: Option<String>,

    /// Federal fiscal year of the award.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fiscal_year: Option<i64>,

    /// GSDM assistance type of the award's latest action.
    /// Shaped as the bare leaf `assistance_type` the API sends only the code; that decodes here with [`description`](AssistanceTypeCode::description) unset.
    #[serde(
        default,
        deserialize_with = "code_or_object",
        skip_serializing_if = "Option::is_none"
    )]
    pub assistance_type: Option<AssistanceTypeCode>,

    /// Record type: `1` aggregate, `2` non-aggregate, `3` non-aggregate to an individual (PII-redacted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_type: Option<String>,

    /// The award's primary assistance listing (CFDA) number, e.g. `93.778`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cfda_number: Option<String>,

    /// The primary assistance listing's number and title from the catalog; null when the catalog does not list the number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assistance_listing: Option<AssistanceListingSummary>,

    /// Short award title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Federal obligation in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub obligated: Option<f64>,

    /// The obligation for grants and payments and the subsidy cost for loans, in dollars: the one amount comparable across the four assistance groups.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub pragmatic_obligations: Option<f64>,

    /// Face value of a direct loan or loan guarantee, in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub face_value_of_loan: Option<f64>,

    /// Estimated long-term cost of a loan to the government, in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub loan_subsidy_cost: Option<f64>,

    /// The office that made the award.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub awarding_office: Option<AssistanceOffice>,

    /// The office that funded the award.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funding_office: Option<AssistanceOffice>,

    /// The recipient record; null on aggregate and PII-redacted records (record types `1` and `3`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient: Option<AssistanceRecipient>,

    /// Where the work is done.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place_of_performance: Option<AssistancePlaceOfPerformance>,

    /// When the work runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period_of_performance: Option<AssistancePeriodOfPerformance>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl AssistanceAwardRecord {
    /// Decode a row from a `list_assistance_*` page or `iterate_assistance_*` stream.
    pub fn from_record(record: &Record) -> serde_json::Result<Self> {
        serde_json::from_value(Value::Object(record.clone()))
    }
}

/// The `assistance_type(code,description)` expand on an [`AssistanceAwardRecord`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AssistanceTypeCode {
    /// GSDM assistance type code, `F001` through `F010`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// The code's description, e.g. `Grant`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// The `assistance_listing(number,title)` expand on an [`AssistanceAwardRecord`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AssistanceListingSummary {
    /// Assistance listing (CFDA) number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,

    /// Program title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// The `awarding_office(*)` and `funding_office(*)` expands on an [`AssistanceAwardRecord`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AssistanceOffice {
    /// Tango organization UUID of the office.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,

    /// Office code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub office_code: Option<String>,

    /// Office name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub office_name: Option<String>,

    /// Agency code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agency_code: Option<String>,

    /// Agency name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agency_name: Option<String>,

    /// Department code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub department_code: Option<String>,

    /// Department name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub department_name: Option<String>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// The `recipient(*)` expand on an [`AssistanceAwardRecord`]: a shared recipient record shaped like an entity.
///
/// Every assistance recipient has one, including recipients with no UEI, and its values are the latest the recipient filed.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AssistanceRecipient {
    /// Tango's recipient identifier; for a recipient with a UEI it is the entity's UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,

    /// Unique Entity Identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uei: Option<String>,

    /// Legacy DUNS number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duns: Option<String>,

    /// Normalized recipient name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_business_name: Option<String>,

    /// Recipient name as filed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_business_name_raw: Option<String>,

    /// County of the recipient's address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub county: Option<String>,

    /// The recipient's current congressional district, e.g. `MD-01`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub congressional_district: Option<String>,

    /// The congressional district as filed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub congressional_district_original: Option<String>,

    /// The recipient's highest-level owner, when reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highest_owner: Option<AssistanceRecipientOwner>,

    /// USAspending recipient-type codes (not SAM.gov's), each with its description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_types: Option<Vec<AssistanceBusinessType>>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// The `highest_owner` of an [`AssistanceRecipient`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AssistanceRecipientOwner {
    /// The owner's UEI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uei: Option<String>,

    /// The owner's legacy DUNS number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duns: Option<String>,

    /// Normalized owner name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_business_name: Option<String>,

    /// Owner name as filed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_business_name_raw: Option<String>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// One element of an [`AssistanceRecipient`]'s `business_types`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AssistanceBusinessType {
    /// USAspending recipient-type code, e.g. `A`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// The code's description, e.g. `State Government`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// The `place_of_performance(*)` expand on an [`AssistanceAwardRecord`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AssistancePlaceOfPerformance {
    /// ISO 3166 alpha-3 country code, e.g. `USA`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// State code, e.g. `VA`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_code: Option<String>,

    /// State name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_name: Option<String>,

    /// State FIPS code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_fips: Option<String>,

    /// County FIPS code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub county_fips: Option<String>,

    /// County name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub county_name: Option<String>,

    /// City name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city_name: Option<String>,

    /// Five-digit ZIP code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zip_code: Option<String>,

    /// ZIP+4 suffix.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zip_plus_four: Option<String>,

    /// Place-of-performance code as filed, e.g. `AL36633`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Scope of the place of performance, e.g. `SINGLE ZIP CODE`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Free-text location outside the United States.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foreign_location: Option<String>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// The `period_of_performance(start_date,current_end_date)` expand on an [`AssistanceAwardRecord`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AssistancePeriodOfPerformance {
    /// ISO start date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,

    /// ISO current end date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_end_date: Option<String>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// One action on an assistance award, from its `/transactions/` sub-route (latest first).
///
/// Returned in pages by [`Client::list_assistance_grant_transactions`](crate::Client::list_assistance_grant_transactions) and its loan, direct-payment and other-assistance siblings.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AssistanceTransaction {
    /// The transaction's own key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Modification number; null on the base action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modification_number: Option<String>,

    /// ISO date of the action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_date: Option<String>,

    /// Action type code, e.g. `A1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_type: Option<String>,

    /// GSDM assistance type code of the action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assistance_type_code: Option<String>,

    /// Correction or delete indicator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction_delete_indicator: Option<String>,

    /// Assistance listing (CFDA) number of the action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cfda_number: Option<String>,

    /// Federal obligation of the action, in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub obligated: Option<f64>,

    /// The action's obligation for grants and payments and its subsidy cost for loans, in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub pragmatic_obligations: Option<f64>,

    /// The recipient's cost share, in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub non_federal_funding_amount: Option<f64>,

    /// Federal share of indirect costs, in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub indirect_cost_federal_share_amount: Option<f64>,

    /// Face value of a loan, in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub face_value_of_loan: Option<f64>,

    /// Loan subsidy cost, in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub loan_subsidy_cost: Option<f64>,

    /// Award amount basis code, e.g. `A01`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub award_amount_basis: Option<String>,

    /// Award recipient basis code, e.g. `R01`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub award_recipient_basis: Option<String>,

    /// Federal fiscal year of the action date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_date_fiscal_year: Option<i64>,

    /// Timestamp the action was last modified at the source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<String>,

    /// Timestamp the action was first reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_report_date: Option<String>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// An assistance award's money under one assistance listing (CFDA), from its `/listings/` sub-route (primary listing first).
///
/// Returned in pages by [`Client::list_assistance_grant_listings`](crate::Client::list_assistance_grant_listings) and its loan, direct-payment and other-assistance siblings.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AssistanceAwardListing {
    /// Assistance listing (CFDA) number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cfda_number: Option<String>,

    /// Whether this is the award's primary listing, the one the `cfda_number` filter matches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_primary: Option<bool>,

    /// Federal obligation under this listing, in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub obligated: Option<f64>,

    /// Obligation for grants and payments and subsidy cost for loans under this listing, in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub pragmatic_obligations: Option<f64>,

    /// The recipient's cost share under this listing, in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub non_federal_funding_amount: Option<f64>,

    /// Federal share of indirect costs under this listing, in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub indirect_cost_federal_share_amount: Option<f64>,

    /// Face value of loans under this listing, in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub face_value_of_loan: Option<f64>,

    /// Loan subsidy cost under this listing, in dollars.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub loan_subsidy_cost: Option<f64>,

    /// Number of the award's transactions under this listing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transactions_count: Option<i64>,

    /// ISO date of the first action under this listing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_action_date: Option<String>,

    /// ISO date of the last action under this listing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_action_date: Option<String>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Decode `assistance_type` from the `{code, description}` expand or from the bare code the leaf form sends.
fn code_or_object<'de, D>(deserializer: D) -> Result<Option<AssistanceTypeCode>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<Value>::deserialize(deserializer)? {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(code)) => Ok(Some(AssistanceTypeCode {
            code: Some(code),
            ..Default::default()
        })),
        Some(v @ Value::Object(_)) => serde_json::from_value(v)
            .map(Some)
            .map_err(serde::de::Error::custom),
        Some(other) => Err(serde::de::Error::custom(format!(
            "expected an assistance type code or object, got {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn default_shape_award() -> Value {
        json!({
            "assistance_listing": {"number": "20.823", "title": "Port Infrastructure Development Program"},
            "assistance_type": {"code": "F001", "description": "Grant"},
            "award_date": "2026-10-19",
            "awarding_office": {
                "organization_id": "261f885d-c8b6-5e0d-b2f8-e0799a4e1de2",
                "office_code": "69A334",
                "office_name": "MARITIME ADMINISTRATION",
                "agency_code": "6938",
                "agency_name": "MARITIME ADMINISTRATION",
                "department_code": "069",
                "department_name": "TRANSPORTATION, DEPARTMENT OF"
            },
            "cfda_number": "20.823",
            "face_value_of_loan": 0.0,
            "fain": "693JF72640002",
            "fiscal_year": 2027,
            "funding_office": {"office_code": "69A334", "department_code": "069"},
            "key": "ASST_NON_693JF72640002_069",
            "loan_subsidy_cost": 0.0,
            "obligated": 2_000_000.0,
            "period_of_performance": {"start_date": "2026-10-19", "current_end_date": "2029-04-30"},
            "place_of_performance": {
                "city_name": "MOBILE",
                "code": "AL36633",
                "country": "USA",
                "county_fips": "01097",
                "county_name": "MOBILE",
                "foreign_location": null,
                "scope": "SINGLE ZIP CODE",
                "state_code": "AL",
                "state_fips": "01",
                "state_name": "ALABAMA",
                "zip_code": "36633",
                "zip_plus_four": "1588"
            },
            "pragmatic_obligations": 2_000_000.0,
            "recipient": {
                "uuid": "510bd6cc-9840-5adb-98d1-d8e5f74dff6b",
                "uei": "EKPQAJKKV549",
                "duns": null,
                "legal_business_name": "CITY OF CRISFIELD",
                "legal_business_name_raw": "CITY OF CRISFIELD",
                "county": "SOMERSET",
                "congressional_district": "MD-01",
                "congressional_district_original": "MD-01",
                "highest_owner": {
                    "uei": null,
                    "duns": null,
                    "legal_business_name": null,
                    "legal_business_name_raw": "CITY OF CRISFIELD"
                },
                "business_types": [{"code": "A", "description": "State Government"}]
            },
            "record_type": "2",
            "title": "AWARD PURPOSE THE PROJECT INCLUDES ELECTRICAL…",
            "uri": null,
            "former_award_keys": ["ASST_NON_OLD_069"]
        })
    }

    #[test]
    fn award_decodes_every_default_shape_field() {
        let rec: AssistanceAwardRecord =
            serde_json::from_value(default_shape_award()).expect("decode");
        assert_eq!(rec.key.as_deref(), Some("ASST_NON_693JF72640002_069"));
        assert_eq!(rec.fain.as_deref(), Some("693JF72640002"));
        assert_eq!(rec.uri, None);
        assert_eq!(rec.award_date.as_deref(), Some("2026-10-19"));
        assert_eq!(rec.fiscal_year, Some(2027));
        let kind = rec.assistance_type.as_ref().expect("assistance_type");
        assert_eq!(kind.code.as_deref(), Some("F001"));
        assert_eq!(kind.description.as_deref(), Some("Grant"));
        assert_eq!(rec.record_type.as_deref(), Some("2"));
        assert_eq!(rec.cfda_number.as_deref(), Some("20.823"));
        assert_eq!(
            rec.assistance_listing
                .as_ref()
                .and_then(|l| l.title.as_deref()),
            Some("Port Infrastructure Development Program")
        );
        assert_eq!(
            rec.title.as_deref(),
            Some("AWARD PURPOSE THE PROJECT INCLUDES ELECTRICAL…")
        );
        assert_eq!(rec.obligated, Some(2_000_000.0));
        assert_eq!(rec.pragmatic_obligations, Some(2_000_000.0));
        assert_eq!(rec.face_value_of_loan, Some(0.0));
        assert_eq!(rec.loan_subsidy_cost, Some(0.0));
        let office = rec.awarding_office.as_ref().expect("awarding_office");
        assert_eq!(
            office.organization_id.as_deref(),
            Some("261f885d-c8b6-5e0d-b2f8-e0799a4e1de2")
        );
        assert_eq!(
            office.department_name.as_deref(),
            Some("TRANSPORTATION, DEPARTMENT OF")
        );
        assert_eq!(
            rec.funding_office
                .as_ref()
                .and_then(|o| o.office_code.as_deref()),
            Some("69A334")
        );
        let recipient = rec.recipient.as_ref().expect("recipient");
        assert_eq!(recipient.uei.as_deref(), Some("EKPQAJKKV549"));
        assert_eq!(recipient.duns, None);
        assert_eq!(recipient.congressional_district.as_deref(), Some("MD-01"));
        assert_eq!(
            recipient
                .highest_owner
                .as_ref()
                .and_then(|o| o.legal_business_name_raw.as_deref()),
            Some("CITY OF CRISFIELD")
        );
        let types = recipient.business_types.as_deref().expect("business_types");
        assert_eq!(types[0].code.as_deref(), Some("A"));
        assert_eq!(types[0].description.as_deref(), Some("State Government"));
        let place = rec.place_of_performance.as_ref().expect("place");
        assert_eq!(place.state_code.as_deref(), Some("AL"));
        assert_eq!(place.zip_plus_four.as_deref(), Some("1588"));
        assert_eq!(place.foreign_location, None);
        let period = rec.period_of_performance.as_ref().expect("period");
        assert_eq!(period.current_end_date.as_deref(), Some("2029-04-30"));
        assert_eq!(
            rec.extra.get("former_award_keys"),
            Some(&json!(["ASST_NON_OLD_069"]))
        );
    }

    #[test]
    fn assistance_type_leaf_decodes_as_a_bare_code() {
        let rec: AssistanceAwardRecord =
            serde_json::from_value(json!({"key": "K", "assistance_type": "F003"})).expect("decode");
        let kind = rec.assistance_type.expect("assistance_type");
        assert_eq!(kind.code.as_deref(), Some("F003"));
        assert_eq!(kind.description, None);
    }

    #[test]
    fn null_recipient_on_an_aggregate_record_is_none() {
        let rec: AssistanceAwardRecord = serde_json::from_value(
            json!({"key": "K", "record_type": "1", "recipient": null, "assistance_type": null}),
        )
        .expect("decode");
        assert_eq!(rec.recipient, None);
        assert_eq!(rec.assistance_type, None);
    }

    #[test]
    fn from_record_decodes_a_list_row() {
        let row = default_shape_award().as_object().cloned().expect("object");
        let rec = AssistanceAwardRecord::from_record(&row).expect("decode");
        assert_eq!(rec.key.as_deref(), Some("ASST_NON_693JF72640002_069"));
    }

    #[test]
    fn award_round_trips_without_reintroducing_absent_keys() {
        let rec = AssistanceAwardRecord {
            key: Some("K".into()),
            obligated: Some(1.5),
            ..Default::default()
        };
        let value = serde_json::to_value(&rec).expect("serialize");
        assert_eq!(value, json!({"key": "K", "obligated": 1.5}));
    }

    #[test]
    fn transaction_decodes_every_field() {
        let tx: AssistanceTransaction = serde_json::from_value(json!({
            "key": "6938_693JF72640002_-NONE-_20.823_-NONE-",
            "modification_number": null,
            "transaction_date": "2026-10-19",
            "action_type": "A1",
            "assistance_type_code": "F001",
            "correction_delete_indicator": null,
            "cfda_number": "20.823",
            "obligated": 2_000_000.0,
            "pragmatic_obligations": 2_000_000.0,
            "non_federal_funding_amount": 500_000.0,
            "indirect_cost_federal_share_amount": 0.0,
            "face_value_of_loan": 0.0,
            "loan_subsidy_cost": "12.50",
            "award_amount_basis": "A01",
            "award_recipient_basis": "R01",
            "action_date_fiscal_year": 2027,
            "last_modified_date": "2026-10-20T13:59:38Z",
            "initial_report_date": "2026-10-20T13:59:38Z"
        }))
        .expect("decode");
        assert_eq!(
            tx.key.as_deref(),
            Some("6938_693JF72640002_-NONE-_20.823_-NONE-")
        );
        assert_eq!(tx.modification_number, None);
        assert_eq!(tx.action_type.as_deref(), Some("A1"));
        assert_eq!(tx.assistance_type_code.as_deref(), Some("F001"));
        assert_eq!(tx.non_federal_funding_amount, Some(500_000.0));
        assert_eq!(tx.loan_subsidy_cost, Some(12.5));
        assert_eq!(tx.award_recipient_basis.as_deref(), Some("R01"));
        assert_eq!(tx.action_date_fiscal_year, Some(2027));
        assert_eq!(
            tx.initial_report_date.as_deref(),
            Some("2026-10-20T13:59:38Z")
        );
        assert!(tx.extra.is_empty());
    }

    #[test]
    fn listing_decodes_every_field() {
        let listing: AssistanceAwardListing = serde_json::from_value(json!({
            "cfda_number": "20.823",
            "is_primary": true,
            "obligated": 2_000_000.0,
            "pragmatic_obligations": 2_000_000.0,
            "non_federal_funding_amount": 500_000.0,
            "indirect_cost_federal_share_amount": 0.0,
            "face_value_of_loan": 0.0,
            "loan_subsidy_cost": 0.0,
            "transactions_count": 1,
            "first_action_date": "2026-10-19",
            "last_action_date": "2026-10-19"
        }))
        .expect("decode");
        assert_eq!(listing.cfda_number.as_deref(), Some("20.823"));
        assert_eq!(listing.is_primary, Some(true));
        assert_eq!(listing.obligated, Some(2_000_000.0));
        assert_eq!(listing.transactions_count, Some(1));
        assert_eq!(listing.last_action_date.as_deref(), Some("2026-10-19"));
        assert!(listing.extra.is_empty());
    }
}
