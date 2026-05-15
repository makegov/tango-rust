//! Error types returned by the SDK.
//!
//! Every fallible call returns [`Result<T, Error>`](crate::Result). The
//! [`Error`] enum carries enough structure that callers can dispatch on the
//! variant (e.g. retry on `RateLimit`, surface `Validation.message` to a user)
//! without parsing strings. For programmatic dispatch by HTTP status,
//! [`Error::status`] returns `Some(code)` for variants that have one.
//!
//! The retry loop in [`crate::transport`] calls [`Error::is_retryable`] to
//! decide whether to wait and re-issue the request.

use std::time::Duration;

/// A parsed Tango API error body. Carried on the API-error variants so callers
/// can introspect the server's structured error response without re-parsing.
///
/// The Tango API surfaces error bodies in a handful of shapes:
///
/// - `{"detail": "..."}` (DRF envelope)
/// - `{"message": "..."}` or `{"error": "..."}` (generic envelope)
/// - `{"<field>": ["..."]}` (DRF field-error array)
///
/// The raw decoded JSON is preserved in [`ErrorBody::raw`] for callers that
/// need to inspect a custom shape. [`ErrorBody::message`] is the SDK's best
/// guess at a human-readable line — extracted by walking envelope keys
/// (`detail` / `message` / `error`) first, then sorted-key iteration over the
/// remaining keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorBody {
    /// The human-readable message the SDK extracted from the body, when one
    /// was available. Empty when the body had no obvious message slot.
    pub message: String,

    /// The raw decoded JSON value as a string (re-serialized to be stable).
    /// `None` when the response had no body or the body was not JSON.
    pub raw: Option<serde_json::Value>,
}

/// The error type returned by all fallible SDK calls.
///
/// Use [`Error::is_retryable`] in custom retry policies and [`Error::status`]
/// for programmatic dispatch by HTTP status code.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// HTTP 401 — the API key was missing, malformed, or rejected.
    #[error("tango: authentication failed (status 401)")]
    Auth {
        /// The parsed error body, when one was returned.
        response: Option<ErrorBody>,
    },

    /// HTTP 404 — the requested resource does not exist.
    #[error("tango: resource not found (status 404)")]
    NotFound {
        /// The parsed error body, when one was returned.
        response: Option<ErrorBody>,
    },

    /// HTTP 400 — the request was syntactically valid but the server rejected
    /// its parameters. The `message` is the SDK's best guess at a human-readable
    /// reason; the SDK walks envelope keys (`detail` / `message` / `error`)
    /// first, then sorted-key iteration over the remaining keys, preferring
    /// array values over strings.
    #[error("tango: invalid request (status 400): {message}")]
    Validation {
        /// Human-readable validation message.
        message: String,
        /// The parsed error body, when one was returned.
        response: Option<ErrorBody>,
    },

    /// HTTP 429 — the caller exceeded a rate limit.
    ///
    /// `retry_after` is populated from the `Retry-After` header when present
    /// (in seconds, capped at 10s by the retry loop). `limit_type` is
    /// populated from `X-RateLimit-Type` when the server sets it
    /// (e.g. `"minute"`, `"hour"`, `"day"`).
    #[error("tango: rate limit exceeded (status 429); retry after {retry_after}s")]
    RateLimit {
        /// Seconds to wait before retrying, as suggested by the server.
        retry_after: u32,
        /// The rate-limit bucket the server reported, when present.
        limit_type: Option<String>,
        /// The parsed error body, when one was returned.
        response: Option<ErrorBody>,
    },

    /// A request exceeded its configured timeout. No HTTP status was received.
    #[error("tango: request timed out after {timeout:?}")]
    Timeout {
        /// The timeout duration that elapsed.
        timeout: Duration,
    },

    /// Any other non-2xx HTTP response.
    #[error("tango: API error (status {status}): {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Human-readable message.
        message: String,
        /// The parsed error body, when one was returned.
        response: Option<ErrorBody>,
    },

    /// An HTTP transport-level failure (DNS, connection refused, TLS, etc.).
    /// `Send`-safe `reqwest::Error` is preserved for `Error::source` traversal.
    #[error("tango: HTTP transport error")]
    Transport(#[from] reqwest::Error),

    /// A response body failed JSON decoding.
    #[error("tango: failed to decode response body")]
    Decode(#[from] serde_json::Error),

    /// The SDK failed to build a request (URL parse, bad input, etc.).
    /// Strictly internal — callers should not see this in practice.
    #[error("tango: failed to build request: {0}")]
    Build(String),
}

impl Error {
    /// Returns the HTTP status code for variants that carry one.
    #[must_use]
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Auth { .. } => Some(401),
            Self::NotFound { .. } => Some(404),
            Self::Validation { .. } => Some(400),
            Self::RateLimit { .. } => Some(429),
            Self::Api { status, .. } => Some(*status),
            Self::Timeout { .. } | Self::Transport(_) | Self::Decode(_) | Self::Build(_) => None,
        }
    }

    /// Returns `true` if this error is one the SDK's retry loop will recover from.
    ///
    /// Retryable:
    /// - `RateLimit` (429)
    /// - `Timeout` (transport-level deadline)
    /// - `Api { status: 5xx }` (server-side faults)
    /// - `Api { status: 408 }` (request timeout reported by server)
    /// - `Transport` (DNS/TCP/TLS-level failures)
    ///
    /// Not retryable:
    /// - `Auth` / `NotFound` / `Validation` (client-side; retry won't change the answer)
    /// - `Api { status: other 4xx }`
    /// - `Decode` / `Build` (programmer/server error)
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::RateLimit { .. } | Self::Timeout { .. } | Self::Transport(_) => true,
            Self::Api { status, .. } => *status == 408 || (500..600).contains(status),
            Self::Auth { .. }
            | Self::NotFound { .. }
            | Self::Validation { .. }
            | Self::Decode(_)
            | Self::Build(_) => false,
        }
    }

    /// Returns the parsed error body when one was returned by the server.
    #[must_use]
    pub fn response(&self) -> Option<&ErrorBody> {
        match self {
            Self::Auth { response, .. }
            | Self::NotFound { response, .. }
            | Self::Validation { response, .. }
            | Self::RateLimit { response, .. }
            | Self::Api { response, .. } => response.as_ref(),
            Self::Timeout { .. } | Self::Transport(_) | Self::Decode(_) | Self::Build(_) => None,
        }
    }
}

/// Alias for the SDK's standard `Result` type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_codes() {
        assert_eq!(Error::Auth { response: None }.status(), Some(401));
        assert_eq!(Error::NotFound { response: None }.status(), Some(404));
        assert_eq!(
            Error::Validation {
                message: "x".into(),
                response: None
            }
            .status(),
            Some(400)
        );
        assert_eq!(
            Error::RateLimit {
                retry_after: 1,
                limit_type: None,
                response: None
            }
            .status(),
            Some(429)
        );
        assert_eq!(
            Error::Api {
                status: 502,
                message: "x".into(),
                response: None
            }
            .status(),
            Some(502)
        );
        assert_eq!(
            Error::Timeout {
                timeout: Duration::from_secs(1)
            }
            .status(),
            None
        );
    }

    #[test]
    fn retry_decisions() {
        assert!(Error::RateLimit {
            retry_after: 0,
            limit_type: None,
            response: None
        }
        .is_retryable());
        assert!(Error::Timeout {
            timeout: Duration::from_secs(1)
        }
        .is_retryable());
        assert!(Error::Api {
            status: 502,
            message: "x".into(),
            response: None
        }
        .is_retryable());
        assert!(Error::Api {
            status: 408,
            message: "x".into(),
            response: None
        }
        .is_retryable());

        assert!(!Error::Auth { response: None }.is_retryable());
        assert!(!Error::NotFound { response: None }.is_retryable());
        assert!(!Error::Validation {
            message: "x".into(),
            response: None
        }
        .is_retryable());
        assert!(!Error::Api {
            status: 418,
            message: "x".into(),
            response: None
        }
        .is_retryable());
        assert!(!Error::Build("x".into()).is_retryable());
    }
}
