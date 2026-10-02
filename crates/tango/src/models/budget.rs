//! `BudgetSourceAnomaly` — typed view of a budget account's `source_anomalies` list.

use crate::Record;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// One problem found in the source data behind a budget-account row.
///
/// [`Client::list_budget_accounts`](crate::Client::list_budget_accounts) and [`Client::get_budget_account`](crate::Client::get_budget_account) return untyped [`Record`]s; the `source_anomalies` field in them is a JSON list, `[]` when the row is clean.
/// Decode it with [`BudgetSourceAnomaly::from_record`].
///
/// `code` is an open set, so match on the strings you know and treat anything else as a new kind of anomaly.
/// Known codes: `contract_exceeds_obligations` (contract obligations reported above the account's obligations, so `contract_obligated` is capped), `assistance_exceeds_obligations` and `contract_without_obligations` (both flagged, the served values unchanged).
///
/// Every field is optional, and unknown server-side fields fall through to [`extra`](Self::extra).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BudgetSourceAnomaly {
    /// What kind of anomaly this is, e.g. `"contract_exceeds_obligations"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// The row field the anomaly is about, e.g. `"contract_obligated"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,

    /// The field whose value [`field`](Self::field) was checked against, e.g. `"obligated_total"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bound_field: Option<String>,

    /// What was done about it: `"capped"` (the served value was reduced to the bound) or `"flagged"` (served unchanged).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,

    /// The value the source data reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reported_value: Option<f64>,

    /// The value the API serves for [`field`](Self::field).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub served_value: Option<f64>,

    /// Short description of the most likely cause.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub likely_cause: Option<String>,

    /// Every served field the anomaly affects, including values derived from [`field`](Self::field).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affected_fields: Option<Vec<String>>,

    /// Human-readable explanation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// The source rows behind the anomaly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<BudgetSourceAnomalySource>,

    /// Forward-compat catch-all for server-side fields the SDK does not yet model.
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
}

impl BudgetSourceAnomaly {
    /// Decode the `source_anomalies` field of a budget-account [`Record`].
    ///
    /// Returns an empty list when the field is absent (not in the requested shape) or null, and an error when it is present but not a list of anomaly objects.
    pub fn from_record(record: &Record) -> serde_json::Result<Vec<Self>> {
        match record.get("source_anomalies") {
            None | Some(Value::Null) => Ok(Vec::new()),
            Some(v) => serde_json::from_value(v.clone()),
        }
    }
}

/// Where a [`BudgetSourceAnomaly`] came from.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BudgetSourceAnomalySource {
    /// The source dataset the rows were read from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset: Option<String>,

    /// Fiscal year of the source rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fiscal_year: Option<i64>,

    /// The source rows that account for the anomaly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<BudgetSourceAnomalyRow>>,

    /// Forward-compat catch-all for server-side fields the SDK does not yet model.
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
}

/// One source row behind a [`BudgetSourceAnomaly`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BudgetSourceAnomalyRow {
    /// Fiscal period (month of the fiscal year) the row was reported in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fiscal_period: Option<i64>,

    /// Award PIID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub piid: Option<String>,

    /// Parent award PIID, for an order under an IDV.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_piid: Option<String>,

    /// Treasury Account Symbol the row was reported against.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tas: Option<String>,

    /// Reporting agency identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reporting_agency_id: Option<String>,

    /// Obligated amount the row reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_obligated_amount: Option<f64>,

    /// Which source file the row came from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_c_source: Option<String>,

    /// Forward-compat catch-all for server-side fields the SDK does not yet model.
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn record(v: Value) -> Record {
        v.as_object().cloned().expect("object")
    }

    #[test]
    fn decodes_a_capped_contract_anomaly_with_source_rows() {
        let r = record(json!({
            "federal_account_symbol": "020-4159",
            "source_anomalies": [{
                "code": "contract_exceeds_obligations",
                "field": "contract_obligated",
                "bound_field": "obligated_total",
                "action": "capped",
                "reported_value": 73_470_455_401.8,
                "served_value": 4_748_670_061.77,
                "likely_cause": "unit error",
                "affected_fields": ["contract_obligated", "contract_share_of_obligated_capped"],
                "message": "reported contracts exceed obligations",
                "new_key": 1,
                "source": {
                    "dataset": "file_c",
                    "fiscal_year": 2025,
                    "rows": [{
                        "fiscal_period": 6,
                        "piid": "P1",
                        "parent_piid": null,
                        "tas": "020-2025/2025-4159",
                        "reporting_agency_id": "020",
                        "transaction_obligated_amount": 68_721_785_340.03,
                        "file_c_source": "award_financial"
                    }]
                }
            }]
        }));
        let got = BudgetSourceAnomaly::from_record(&r).expect("decode");
        assert_eq!(got.len(), 1);
        let a = &got[0];
        assert_eq!(a.code.as_deref(), Some("contract_exceeds_obligations"));
        assert_eq!(a.action.as_deref(), Some("capped"));
        assert_eq!(a.reported_value, Some(73_470_455_401.8));
        assert_eq!(a.served_value, Some(4_748_670_061.77));
        assert_eq!(a.affected_fields.as_ref().map(Vec::len), Some(2));
        assert_eq!(a.extra.get("new_key"), Some(&json!(1)));
        let src = a.source.as_ref().expect("source");
        assert_eq!(src.fiscal_year, Some(2025));
        let row = &src.rows.as_ref().expect("rows")[0];
        assert_eq!(row.fiscal_period, Some(6));
        assert_eq!(row.parent_piid, None);
        assert_eq!(row.reporting_agency_id.as_deref(), Some("020"));
        assert_eq!(row.transaction_obligated_amount, Some(68_721_785_340.03));
    }

    #[test]
    fn sparse_anomaly_and_unknown_code_decode() {
        let r = record(json!({"source_anomalies": [{"code": "something_new"}]}));
        let got = BudgetSourceAnomaly::from_record(&r).expect("decode");
        assert_eq!(got[0].code.as_deref(), Some("something_new"));
        assert!(got[0].source.is_none());
    }

    #[test]
    fn clean_absent_and_null_are_empty() {
        for v in [
            json!({"source_anomalies": []}),
            json!({}),
            json!({"source_anomalies": null}),
        ] {
            assert!(BudgetSourceAnomaly::from_record(&record(v))
                .expect("decode")
                .is_empty());
        }
    }

    #[test]
    fn non_list_is_an_error() {
        let r = record(json!({"source_anomalies": "oops"}));
        assert!(BudgetSourceAnomaly::from_record(&r).is_err());
    }
}
