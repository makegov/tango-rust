//! `FiscalYearObligation` — typed view of the `obligations_by_fiscal_year` shape expand on contracts, IDVs, OTAs and OT IDVs.

use crate::Record;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// One federal fiscal year of an award's obligations, from the `obligations_by_fiscal_year(fiscal_year,obligated)` shape expand (Tango API 5.11.0).
///
/// The expand sums the award's transactions per federal fiscal year (October 1 starts the next fiscal year), oldest year first, each rounded to cents.
/// A de-obligation nets against its own year, so a year can be zero or negative.
/// It is returned only when named in `shape`: no default shape includes it.
/// Decode it from a contract, IDV, OTA or OT IDV record with [`FiscalYearObligation::from_record`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FiscalYearObligation {
    /// The federal fiscal year, e.g. `2025` for October 2024 through September 2025.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fiscal_year: Option<i64>,

    /// Net obligations in that fiscal year, in dollars, rounded to cents.
    #[serde(
        default,
        deserialize_with = "crate::models::budget::number_or_numeric_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub obligated: Option<f64>,

    /// Forward-compat catch-all for server-side fields the SDK does not yet model.
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
}

impl FiscalYearObligation {
    /// Decode the `obligations_by_fiscal_year` field of an award [`Record`].
    ///
    /// Returns `Ok(None)` when the field is absent (not in the requested shape) or null.
    /// It is always null on IDVs and OT IDVs, whose transactions report a running lifetime total rather than each action's obligation, and null on a contract or OTA with no dated transactions.
    /// Returns an error when the field is present but not a list of year objects.
    pub fn from_record(record: &Record) -> serde_json::Result<Option<Vec<Self>>> {
        match record.get("obligations_by_fiscal_year") {
            None | Some(Value::Null) => Ok(None),
            Some(v) => serde_json::from_value(v.clone()).map(Some),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn record(v: Value) -> Record {
        v.as_object().cloned().expect("object")
    }

    #[test]
    fn decodes_years_in_order_including_a_negative_year() {
        let r = record(json!({
            "key": "CONT_AWD_1",
            "obligations_by_fiscal_year": [
                {"fiscal_year": 2024, "obligated": 125_000.5},
                {"fiscal_year": 2025, "obligated": -1_200.0},
                {"fiscal_year": 2026, "obligated": "300.25"}
            ]
        }));
        let years = FiscalYearObligation::from_record(&r)
            .expect("decode")
            .expect("present");
        assert_eq!(years.len(), 3);
        assert_eq!(years[0].fiscal_year, Some(2024));
        assert_eq!(years[0].obligated, Some(125_000.5));
        assert_eq!(years[1].obligated, Some(-1_200.0));
        assert_eq!(years[2].obligated, Some(300.25));
    }

    #[test]
    fn null_and_absent_are_none() {
        for v in [json!({"obligations_by_fiscal_year": null}), json!({})] {
            assert!(FiscalYearObligation::from_record(&record(v))
                .expect("decode")
                .is_none());
        }
    }

    #[test]
    fn non_list_is_an_error() {
        let r = record(json!({"obligations_by_fiscal_year": {"fiscal_year": 2025}}));
        assert!(FiscalYearObligation::from_record(&r).is_err());
    }
}
