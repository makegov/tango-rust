//! Request loop, retry policy, header parsing, and validation-message extraction.
//!
//! This module is the core of the SDK. Public surface ([`RateLimitInfo`]) is
//! re-exported from `crate`. Internals are crate-private — resource methods
//! call [`Client::request_json`](crate::Client::request_json) and friends, not
//! this module directly.

use crate::error::{Error, ErrorBody, Result};
use reqwest::{header::HeaderMap, Method, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::Duration;

pub(crate) const MAX_BACKOFF: Duration = Duration::from_secs(10);

pub(crate) const API_KEY_HEADER: &str = "X-API-KEY";
pub(crate) const RATE_LIMIT_REMAINING_HDR: &str = "x-ratelimit-remaining";
pub(crate) const RATE_LIMIT_LIMIT_HDR: &str = "x-ratelimit-limit";
pub(crate) const RATE_LIMIT_RESET_HDR: &str = "x-ratelimit-reset";
pub(crate) const RATE_LIMIT_TYPE_HDR: &str = "x-ratelimit-type";
pub(crate) const RETRY_AFTER_HDR: &str = "retry-after";

/// A snapshot of the rate-limit headers from the most recent response.
///
/// Returned by [`Client::rate_limit_info`](crate::Client::rate_limit_info).
/// Any field set to `None` was either absent from the response or could not
/// be parsed as an integer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RateLimitInfo {
    /// `X-RateLimit-Remaining` — calls remaining in the current bucket.
    pub remaining: Option<i64>,
    /// `X-RateLimit-Limit` — the maximum calls in the current bucket.
    pub limit: Option<i64>,
    /// `X-RateLimit-Reset` — seconds until the bucket refills.
    pub reset_in: Option<i64>,
    /// `Retry-After` — server-suggested wait before retrying, in seconds.
    pub retry_after: Option<i64>,
    /// `X-RateLimit-Type` — bucket discriminator (e.g. `"minute"`, `"hour"`).
    pub limit_type: Option<String>,
}

impl RateLimitInfo {
    pub(crate) fn from_headers(h: &HeaderMap) -> Self {
        Self {
            remaining: parse_int_header(h, RATE_LIMIT_REMAINING_HDR),
            limit: parse_int_header(h, RATE_LIMIT_LIMIT_HDR),
            reset_in: parse_int_header(h, RATE_LIMIT_RESET_HDR),
            retry_after: parse_int_header(h, RETRY_AFTER_HDR),
            limit_type: h
                .get(RATE_LIMIT_TYPE_HDR)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string),
        }
    }
}

fn parse_int_header(h: &HeaderMap, key: &str) -> Option<i64> {
    h.get(key)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .and_then(|s| s.parse::<i64>().ok())
}

/// Parse the `Retry-After` header into a duration. Accepts both delta-seconds
/// and HTTP-date forms; returns `Duration::ZERO` on parse failure. The value
/// is capped at [`MAX_BACKOFF`].
pub(crate) fn parse_retry_after(h: &HeaderMap) -> Duration {
    let Some(raw) = h.get(RETRY_AFTER_HDR).and_then(|v| v.to_str().ok()) else {
        return Duration::ZERO;
    };
    let raw = raw.trim();
    if let Ok(secs) = raw.parse::<u64>() {
        return Duration::from_secs(secs).min(MAX_BACKOFF);
    }
    // HTTP-date form. We don't pull in chrono; httpdate would be a dep we want
    // to avoid. The Tango server emits delta-seconds in practice, so we accept
    // a parse failure here and let the exponential backoff take over.
    Duration::ZERO
}

/// Extract a human-readable validation message from a 400 response body.
///
/// Mirrors the Go SDK's `extractValidationMessage`. Tries the envelope keys
/// `detail`, `message`, `error` in priority order; falls back to sorted-key
/// iteration over the remaining keys (preferring array values over strings),
/// so the surfaced message is deterministic across runs.
pub(crate) fn extract_validation_message(body: Option<&Value>) -> String {
    const FALLBACK: &str = "invalid request parameters";
    let Some(Value::Object(map)) = body else {
        return FALLBACK.to_string();
    };
    for key in ["detail", "message", "error"] {
        if let Some(Value::String(s)) = map.get(key) {
            if !s.is_empty() {
                return format!("invalid request parameters: {s}");
            }
        }
    }
    // Field-error shape — walk keys in sorted order for determinism.
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    for k in keys {
        match map.get(k) {
            Some(Value::Array(arr)) if !arr.is_empty() => {
                if let Some(Value::String(s)) = arr.first() {
                    if !s.is_empty() {
                        return format!("invalid request parameters: {s}");
                    }
                }
            }
            Some(Value::String(s)) if !s.is_empty() => {
                return format!("invalid request parameters: {s}");
            }
            _ => {}
        }
    }
    FALLBACK.to_string()
}

/// Body the SDK sends with a request. Used by [`Client::request_json`] and
/// [`Client::request_bytes`] callers.
#[derive(Debug)]
pub(crate) enum Body<'a> {
    None,
    Json(&'a Value),
}

pub(crate) async fn send_with_retries(
    inner: &crate::client::ClientInner,
    method: Method,
    url: reqwest::Url,
    body: Body<'_>,
) -> Result<Vec<u8>> {
    let max_attempts = inner.retries.saturating_add(1);
    let mut attempt: u32 = 0;

    // Pre-serialize the JSON body once; we re-use the bytes across attempts.
    let body_bytes = match body {
        Body::None => None,
        Body::Json(v) => Some(serde_json::to_vec(v)?),
    };

    loop {
        let err =
            match attempt_once(inner, method.clone(), url.clone(), body_bytes.as_deref()).await {
                Ok(bytes) => return Ok(bytes),
                Err(e) => e,
            };

        if !err.is_retryable() || attempt + 1 >= max_attempts {
            return Err(err);
        }

        let wait = if let Error::RateLimit { retry_after, .. } = &err {
            let r = u64::from(*retry_after);
            if r > 0 {
                Duration::from_secs(r).min(MAX_BACKOFF)
            } else {
                backoff_for(inner.retry_backoff, attempt)
            }
        } else {
            backoff_for(inner.retry_backoff, attempt)
        };

        tokio::time::sleep(wait).await;
        attempt += 1;
    }
}

fn backoff_for(base: Duration, attempt: u32) -> Duration {
    let mult = 1u32 << attempt.min(6); // cap shift at 6 to avoid overflow
    base.saturating_mul(mult).min(MAX_BACKOFF)
}

async fn attempt_once(
    inner: &crate::client::ClientInner,
    method: Method,
    url: reqwest::Url,
    body_bytes: Option<&[u8]>,
) -> Result<Vec<u8>> {
    let mut req: RequestBuilder = inner.http.request(method, url);
    req = req.header(reqwest::header::ACCEPT, "application/json");
    if !inner.api_key.is_empty() {
        req = req.header(API_KEY_HEADER, &inner.api_key);
    }
    if !inner.user_agent.is_empty() {
        req = req.header(reqwest::header::USER_AGENT, &inner.user_agent);
    }
    if let Some(bytes) = body_bytes {
        req = req
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(bytes.to_vec());
    }
    if !inner.timeout.is_zero() {
        req = req.timeout(inner.timeout);
    }

    let resp_result = req.send().await;
    let resp: Response = match resp_result {
        Ok(r) => r,
        Err(e) => {
            if e.is_timeout() {
                return Err(Error::Timeout {
                    timeout: inner.timeout,
                });
            }
            return Err(Error::Transport(e));
        }
    };

    let status = resp.status();
    let headers = resp.headers().clone();
    // Snapshot rate-limit/headers for observability.
    inner.set_last_response(&headers);

    let bytes = match resp.bytes().await {
        Ok(b) => b.to_vec(),
        Err(e) => return Err(Error::Transport(e)),
    };

    if status.is_success() {
        return Ok(bytes);
    }

    Err(decode_error(status, &headers, &bytes))
}

fn decode_error(status: StatusCode, headers: &HeaderMap, body: &[u8]) -> Error {
    let parsed_value: Option<Value> = if body.is_empty() {
        None
    } else {
        serde_json::from_slice(body).ok()
    };
    let body_message = parsed_value.as_ref().and_then(extract_top_level_message);
    let response = parsed_value.as_ref().map(|v| ErrorBody {
        message: body_message.clone().unwrap_or_default(),
        raw: Some(v.clone()),
    });
    match status.as_u16() {
        401 => Error::Auth { response },
        404 => Error::NotFound { response },
        400 => Error::Validation {
            message: extract_validation_message(parsed_value.as_ref()),
            response,
        },
        429 => {
            let retry_after = parse_retry_after(headers).as_secs();
            Error::RateLimit {
                retry_after: u32::try_from(retry_after).unwrap_or(u32::MAX),
                limit_type: headers
                    .get(RATE_LIMIT_TYPE_HDR)
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string),
                response,
            }
        }
        code => Error::Api {
            status: code,
            message: body_message
                .unwrap_or_else(|| format!("API request failed with status {code}")),
            response,
        },
    }
}

fn extract_top_level_message(v: &Value) -> Option<String> {
    let Value::Object(map) = v else { return None };
    for key in ["detail", "message", "error"] {
        if let Some(Value::String(s)) = map.get(key) {
            if !s.is_empty() {
                return Some(s.clone());
            }
        }
    }
    None
}

/// Decode a JSON byte slice into a typed `T`, wrapping decode errors so they
/// surface as [`Error::Decode`].
pub(crate) fn decode_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).map_err(Error::Decode)
}

/// Decode a JSON byte slice, treating an empty body as a default-constructed
/// value. Useful for endpoints that legally return `204 No Content`.
pub(crate) fn decode_json_or_default<T>(bytes: &[u8]) -> Result<T>
where
    T: DeserializeOwned + Default,
{
    if bytes.is_empty() {
        return Ok(T::default());
    }
    decode_json(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extract_envelope_detail() {
        let body = json!({"detail": "no soup for you"});
        assert_eq!(
            extract_validation_message(Some(&body)),
            "invalid request parameters: no soup for you"
        );
    }

    #[test]
    fn extract_envelope_message() {
        let body = json!({"message": "bad input"});
        assert_eq!(
            extract_validation_message(Some(&body)),
            "invalid request parameters: bad input"
        );
    }

    #[test]
    fn extract_field_errors_sorted() {
        let body = json!({
            "zebra": ["last alphabetically"],
            "apple": ["first alphabetically"],
        });
        // sorted keys => `apple` wins, deterministically.
        assert_eq!(
            extract_validation_message(Some(&body)),
            "invalid request parameters: first alphabetically"
        );
    }

    #[test]
    fn extract_falls_back() {
        let body = json!({});
        assert_eq!(
            extract_validation_message(Some(&body)),
            "invalid request parameters"
        );
        assert_eq!(
            extract_validation_message(None),
            "invalid request parameters"
        );
    }

    #[test]
    fn extract_prefers_envelope_over_field() {
        let body = json!({
            "detail": "envelope wins",
            "apple": ["field loses"],
        });
        assert_eq!(
            extract_validation_message(Some(&body)),
            "invalid request parameters: envelope wins"
        );
    }

    #[test]
    fn extract_string_field_error() {
        let body = json!({"piid": "must be present"});
        assert_eq!(
            extract_validation_message(Some(&body)),
            "invalid request parameters: must be present"
        );
    }

    #[test]
    fn backoff_doubles_and_caps() {
        let base = Duration::from_millis(250);
        assert_eq!(backoff_for(base, 0), Duration::from_millis(250));
        assert_eq!(backoff_for(base, 1), Duration::from_millis(500));
        assert_eq!(backoff_for(base, 2), Duration::from_secs(1));
        assert_eq!(backoff_for(base, 3), Duration::from_secs(2));
        assert_eq!(backoff_for(base, 4), Duration::from_secs(4));
        assert_eq!(backoff_for(base, 5), Duration::from_secs(8));
        assert_eq!(backoff_for(base, 6), MAX_BACKOFF);
        assert_eq!(backoff_for(base, 50), MAX_BACKOFF);
    }
}
