//! `POST /api/resolve/` and `POST /api/validate/`.
//!
//! `resolve` fuzzy-matches a free-text name against a catalog (entities or
//! organizations) and returns ranked candidates. `validate` checks whether a
//! given identifier (PIID, solicitation, UEI) is well-formed and known.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::models::{ResolveInput, ResolveResult, ValidateInput, ValidateResult};

impl Client {
    /// `POST /api/resolve/` — fuzzy-match a name to entity or organization
    /// candidates.
    ///
    /// `input.target_type` is an enum and is therefore always valid;
    /// `input.name` must be non-empty (validated client-side).
    pub async fn resolve(&self, input: ResolveInput) -> Result<ResolveResult> {
        if input.name.is_empty() {
            return Err(Error::Validation {
                message: "Resolve: name is required".into(),
                response: None,
            });
        }
        self.post_json::<_, ResolveResult>("/api/resolve/", &input)
            .await
    }

    /// `POST /api/validate/` — check whether an identifier (PIID, solicitation,
    /// or UEI) is well-formed and known.
    pub async fn validate(&self, input: ValidateInput) -> Result<ValidateResult> {
        if input.value.is_empty() {
            return Err(Error::Validation {
                message: "Validate: value is required".into(),
                response: None,
            });
        }
        self.post_json::<_, ValidateResult>("/api/validate/", &input)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ResolveTargetType, ValidateInputType};

    fn client() -> Client {
        // Validation must trip BEFORE any HTTP call, so the unreachable base
        // URL is safe — the request must never be issued.
        Client::builder()
            .api_key("k")
            .base_url("http://localhost:1".to_string())
            .build()
            .expect("build client")
    }

    #[tokio::test]
    async fn resolve_rejects_empty_name() {
        let err = client()
            .resolve(ResolveInput {
                name: String::new(),
                target_type: ResolveTargetType::Entity,
                state: None,
                city: None,
                context: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn validate_rejects_empty_value() {
        let err = client()
            .validate(ValidateInput {
                kind: ValidateInputType::Uei,
                value: String::new(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }
}
