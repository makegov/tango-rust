//! Metrics endpoints — rolling-window metrics keyed by NAICS code,
//! PSC code, or entity UEI.
//!
//! Three owner-scoped paths share a shape:
//!   - `GET /api/naics/{code}/metrics/{months}/{period_grouping}/`
//!   - `GET /api/psc/{code}/metrics/{months}/{period_grouping}/`
//!   - `GET /api/entities/{uei}/metrics/{months}/{period_grouping}/`
//!
//! The entity flavour is defined in [`super::entity_subresources`]; this
//! module provides the NAICS and PSC flavours plus a [`Client::list_metrics`]
//! dispatcher that routes to whichever one the caller wants.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::resources::agencies::urlencoding;
use crate::Record;

/// Owner-type discriminant for [`ListMetricsOptions::owner_type`].
pub const METRICS_OWNER_NAICS: &str = "naics";
/// Owner-type discriminant for [`ListMetricsOptions::owner_type`].
pub const METRICS_OWNER_PSC: &str = "psc";
/// Owner-type discriminant for [`ListMetricsOptions::owner_type`].
pub const METRICS_OWNER_ENTITY: &str = "entity";

/// Options for the [`Client::list_metrics`] dispatcher.
///
/// `owner_type` must be one of [`METRICS_OWNER_NAICS`], [`METRICS_OWNER_PSC`],
/// or [`METRICS_OWNER_ENTITY`]. `owner_id` is the NAICS code, PSC code, or
/// entity UEI; `months` is the rolling-window length (must be > 0);
/// `period_grouping` is the aggregation granularity (typically `"month"`,
/// `"quarter"`, or `"year"`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListMetricsOptions {
    /// One of [`METRICS_OWNER_NAICS`], [`METRICS_OWNER_PSC`], or
    /// [`METRICS_OWNER_ENTITY`].
    pub owner_type: String,
    /// The NAICS code, PSC code, or entity UEI.
    pub owner_id: String,
    /// Rolling-window length in months. Must be > 0.
    pub months: u32,
    /// Aggregation granularity (e.g. `"month"`, `"quarter"`, `"year"`).
    pub period_grouping: String,
}

impl ListMetricsOptions {
    /// Convenience constructor.
    #[must_use]
    pub fn new(
        owner_type: impl Into<String>,
        owner_id: impl Into<String>,
        months: u32,
        period_grouping: impl Into<String>,
    ) -> Self {
        Self {
            owner_type: owner_type.into(),
            owner_id: owner_id.into(),
            months,
            period_grouping: period_grouping.into(),
        }
    }
}

impl Client {
    /// `GET /api/naics/{code}/metrics/{months}/{period_grouping}/`
    ///
    /// Rolling-window metrics keyed by NAICS code.
    pub async fn get_naics_metrics(
        &self,
        code: &str,
        months: u32,
        period_grouping: &str,
    ) -> Result<Record> {
        if code.is_empty() {
            return Err(Error::Validation {
                message: "get_naics_metrics: NAICS code is required".into(),
                response: None,
            });
        }
        if months == 0 {
            return Err(Error::Validation {
                message: "get_naics_metrics: months must be > 0".into(),
                response: None,
            });
        }
        if period_grouping.is_empty() {
            return Err(Error::Validation {
                message: "get_naics_metrics: period_grouping is required".into(),
                response: None,
            });
        }
        let path = format!(
            "/api/naics/{}/metrics/{}/{}/",
            urlencoding(code),
            months,
            urlencoding(period_grouping),
        );
        self.get_json::<Record>(&path, &[]).await
    }

    /// `GET /api/psc/{code}/metrics/{months}/{period_grouping}/`
    ///
    /// Rolling-window metrics keyed by PSC code.
    pub async fn get_psc_metrics(
        &self,
        code: &str,
        months: u32,
        period_grouping: &str,
    ) -> Result<Record> {
        if code.is_empty() {
            return Err(Error::Validation {
                message: "get_psc_metrics: PSC code is required".into(),
                response: None,
            });
        }
        if months == 0 {
            return Err(Error::Validation {
                message: "get_psc_metrics: months must be > 0".into(),
                response: None,
            });
        }
        if period_grouping.is_empty() {
            return Err(Error::Validation {
                message: "get_psc_metrics: period_grouping is required".into(),
                response: None,
            });
        }
        let path = format!(
            "/api/psc/{}/metrics/{}/{}/",
            urlencoding(code),
            months,
            urlencoding(period_grouping),
        );
        self.get_json::<Record>(&path, &[]).await
    }

    /// Convenience dispatcher: route to NAICS / PSC / entity metrics based
    /// on [`ListMetricsOptions::owner_type`].
    ///
    /// Returns [`Error::Validation`] when:
    /// - `owner_id` is empty,
    /// - `months == 0`,
    /// - `period_grouping` is empty, or
    /// - `owner_type` isn't one of the three known discriminants.
    ///
    /// The entity flavour fetches `/api/entities/{uei}/metrics/{months}/{period_grouping}/`
    /// directly here; once Wave A's `entity_subresources` lands the typed
    /// `Client::get_entity_metrics` helper, this dispatcher will collapse to
    /// delegate to it (no public-surface change).
    pub async fn list_metrics(&self, opts: ListMetricsOptions) -> Result<Record> {
        if opts.owner_id.is_empty() {
            return Err(Error::Validation {
                message: "list_metrics: owner_id is required".into(),
                response: None,
            });
        }
        if opts.months == 0 {
            return Err(Error::Validation {
                message: "list_metrics: months must be > 0".into(),
                response: None,
            });
        }
        if opts.period_grouping.is_empty() {
            return Err(Error::Validation {
                message: "list_metrics: period_grouping is required".into(),
                response: None,
            });
        }
        match opts.owner_type.as_str() {
            METRICS_OWNER_NAICS => {
                self.get_naics_metrics(&opts.owner_id, opts.months, &opts.period_grouping)
                    .await
            }
            METRICS_OWNER_PSC => {
                self.get_psc_metrics(&opts.owner_id, opts.months, &opts.period_grouping)
                    .await
            }
            METRICS_OWNER_ENTITY => {
                self.get_entity_metrics(&opts.owner_id, opts.months, &opts.period_grouping)
                    .await
            }
            _ => Err(Error::Validation {
                message: "list_metrics: owner_type must be one of: naics, psc, entity".into(),
                response: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn naics_metrics_empty_code_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let err = c
            .get_naics_metrics("", 12, "month")
            .await
            .expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn naics_metrics_zero_months_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let err = c
            .get_naics_metrics("541512", 0, "month")
            .await
            .expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn naics_metrics_empty_grouping_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let err = c
            .get_naics_metrics("541512", 12, "")
            .await
            .expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn psc_metrics_empty_code_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let err = c
            .get_psc_metrics("", 12, "month")
            .await
            .expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn psc_metrics_zero_months_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let err = c
            .get_psc_metrics("D302", 0, "month")
            .await
            .expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn list_metrics_empty_owner_id_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let opts = ListMetricsOptions::new(METRICS_OWNER_NAICS, "", 12, "month");
        let err = c.list_metrics(opts).await.expect_err("must error");
        let msg = match &err {
            Error::Validation { message, .. } => message.clone(),
            other => panic!("expected Validation, got {other:?}"),
        };
        assert!(msg.contains("owner_id"), "got: {msg}");
    }

    #[tokio::test]
    async fn list_metrics_zero_months_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let opts = ListMetricsOptions::new(METRICS_OWNER_NAICS, "541512", 0, "month");
        let err = c.list_metrics(opts).await.expect_err("must error");
        let msg = match &err {
            Error::Validation { message, .. } => message.clone(),
            other => panic!("expected Validation, got {other:?}"),
        };
        assert!(msg.contains("months"), "got: {msg}");
    }

    #[tokio::test]
    async fn list_metrics_empty_grouping_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let opts = ListMetricsOptions::new(METRICS_OWNER_NAICS, "541512", 12, "");
        let err = c.list_metrics(opts).await.expect_err("must error");
        let msg = match &err {
            Error::Validation { message, .. } => message.clone(),
            other => panic!("expected Validation, got {other:?}"),
        };
        assert!(msg.contains("period_grouping"), "got: {msg}");
    }

    #[tokio::test]
    async fn list_metrics_unknown_owner_is_validation() {
        let c = Client::builder().api_key("k").build().expect("client");
        let opts = ListMetricsOptions::new("bogus", "541512", 12, "month");
        let err = c.list_metrics(opts).await.expect_err("must error");
        let msg = match &err {
            Error::Validation { message, .. } => message.clone(),
            other => panic!("expected Validation, got {other:?}"),
        };
        assert!(msg.contains("owner_type"), "got: {msg}");
    }
}
