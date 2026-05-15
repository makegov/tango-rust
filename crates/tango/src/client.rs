//! [`Client`], its `bon`-derived [`ClientBuilder`], and the inner shared state.
//!
//! Construct a `Client` via [`Client::builder`]. Required fields are enforced
//! at compile time by `bon`'s type-state pattern, so `.build()` cannot be
//! called without an `api_key` (or `TANGO_API_KEY` in the environment).

use crate::error::{Error, Result};
use crate::transport::{self, Body, RateLimitInfo};
use bon::bon;
use reqwest::header::HeaderMap;
use serde::Serialize;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Default per-request timeout when none is set.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
/// Default retry count when none is set.
pub const DEFAULT_RETRIES: u32 = 3;
/// Default initial retry backoff (doubles each attempt up to 10s) when none is set.
pub const DEFAULT_RETRY_BACKOFF: Duration = Duration::from_millis(250);

/// Shared inner state. Held inside an `Arc` so `Client` is cheap to clone and
/// always `Send + Sync`.
pub(crate) struct ClientInner {
    pub(crate) api_key: String,
    pub(crate) base_url: String,
    pub(crate) http: reqwest::Client,
    pub(crate) timeout: Duration,
    pub(crate) retries: u32,
    pub(crate) retry_backoff: Duration,
    pub(crate) user_agent: String,
    rate_limit: RwLock<Option<RateLimitInfo>>,
    last_headers: RwLock<Option<HeaderMap>>,
}

impl std::fmt::Debug for ClientInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientInner")
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .field("retries", &self.retries)
            .field("retry_backoff", &self.retry_backoff)
            .field("user_agent", &self.user_agent)
            .field("api_key", &"<redacted>")
            .finish_non_exhaustive()
    }
}

impl ClientInner {
    pub(crate) fn set_last_response(&self, headers: &HeaderMap) {
        let info = RateLimitInfo::from_headers(headers);
        if let Ok(mut guard) = self.rate_limit.write() {
            *guard = Some(info);
        }
        if let Ok(mut guard) = self.last_headers.write() {
            *guard = Some(headers.clone());
        }
    }
}

/// The Tango API client. Cheap to clone — all state is behind an `Arc`.
///
/// Construct with [`Client::builder`]. Call `.api_key(...)` on the builder, or
/// set the `TANGO_API_KEY` environment variable and call [`Client::from_env`].
///
/// `Client` is `Send + Sync + Clone`. Share one instance across tasks.
///
/// # Example
///
/// ```no_run
/// use tango::Client;
/// # async fn run() -> tango::Result<()> {
/// let client = Client::builder()
///     .api_key("your-api-key")
///     .build()?;
/// let _agency = client.get_agency("9700", None).await?;
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct Client {
    pub(crate) inner: Arc<ClientInner>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("inner", &self.inner)
            .finish()
    }
}

#[bon]
impl Client {
    /// Start building a [`Client`].
    ///
    /// `api_key` is required (compile-time check via `bon`). Everything else
    /// is optional with sensible defaults; see the individual setters.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Build`] if the underlying `reqwest::Client` fails to
    /// construct (rare; usually means a bad rustls/TLS configuration).
    #[builder(finish_fn = build)]
    pub fn new(
        /// The Tango API key. Falls back to `TANGO_API_KEY` if not provided
        /// at the call site and the env var is set. An empty string falls
        /// back too.
        #[builder(into)]
        api_key: Option<String>,
        /// Base URL for the API. Falls back to `TANGO_BASE_URL`, then to
        /// [`crate::DEFAULT_BASE_URL`].
        #[builder(into)]
        base_url: Option<String>,
        /// Per-request timeout. Defaults to 30 seconds. `Duration::ZERO`
        /// disables the deadline.
        timeout: Option<Duration>,
        /// Number of retries on retryable failures (5xx / 408 / 429 /
        /// transport). The first attempt is not counted, so the total
        /// attempt count is `retries + 1`. Defaults to 3.
        retries: Option<u32>,
        /// Initial backoff between retries; doubles each attempt, capped
        /// at 10 seconds. The server's `Retry-After` header overrides
        /// this. Defaults to 250ms.
        retry_backoff: Option<Duration>,
        /// Custom `User-Agent`. Defaults to `tango-rust/<version>`.
        #[builder(into)]
        user_agent: Option<String>,
        /// Custom `reqwest::Client` for proxy/tracing injection. When
        /// supplied, its built-in timeout is ignored — per-request
        /// deadlines are applied here.
        http_client: Option<reqwest::Client>,
    ) -> Result<Self> {
        let api_key = match api_key.filter(|s| !s.is_empty()) {
            Some(k) => k,
            None => std::env::var("TANGO_API_KEY").unwrap_or_default(),
        };
        let base_url = match base_url.filter(|s| !s.is_empty()) {
            Some(u) => u,
            None => std::env::var("TANGO_BASE_URL")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| crate::shapes::DEFAULT_BASE_URL.to_string()),
        };
        let timeout = timeout.unwrap_or(DEFAULT_TIMEOUT);
        let retries = retries.unwrap_or(DEFAULT_RETRIES);
        let retry_backoff = retry_backoff.unwrap_or(DEFAULT_RETRY_BACKOFF);
        let user_agent = user_agent
            .filter(|s| !s.is_empty())
            .unwrap_or_else(default_user_agent);
        let http = match http_client {
            Some(c) => c,
            None => reqwest::Client::builder()
                .build()
                .map_err(|e| Error::Build(format!("build reqwest client: {e}")))?,
        };

        Ok(Self {
            inner: Arc::new(ClientInner {
                api_key,
                base_url,
                http,
                timeout,
                retries,
                retry_backoff,
                user_agent,
                rate_limit: RwLock::new(None),
                last_headers: RwLock::new(None),
            }),
        })
    }
}

fn default_user_agent() -> String {
    format!("tango-rust/{}", crate::VERSION)
}

impl Client {
    /// Construct a client from `TANGO_API_KEY` and (optionally) `TANGO_BASE_URL`
    /// in the environment, with all other settings at their defaults.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Build`] if the underlying HTTP client fails to construct.
    pub fn from_env() -> Result<Self> {
        Self::builder().build()
    }

    /// The resolved base URL the client will hit.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.inner.base_url
    }

    /// A snapshot of the rate-limit headers from the most recent response, or
    /// `None` if no request has completed yet.
    #[must_use]
    pub fn rate_limit_info(&self) -> Option<RateLimitInfo> {
        self.inner.rate_limit.read().ok().and_then(|g| g.clone())
    }

    /// The full response headers from the most recent completed request, or
    /// `None` if no request has completed yet. Useful for inspecting
    /// `X-Request-Id`, `X-Tango-Trace-Id`, etc.
    #[must_use]
    pub fn last_response_headers(&self) -> Option<HeaderMap> {
        self.inner.last_headers.read().ok().and_then(|g| g.clone())
    }

    pub(crate) fn build_url(&self, path: &str, query: &[(String, String)]) -> Result<reqwest::Url> {
        let base = self.inner.base_url.trim_end_matches('/');
        let path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        let mut url = reqwest::Url::parse(&format!("{base}{path}"))
            .map_err(|e| Error::Build(format!("parse url {base}{path}: {e}")))?;
        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (k, v) in query {
                pairs.append_pair(k, v);
            }
        }
        Ok(url)
    }

    /// Internal: GET `path` with `query`, decode response as `T`.
    pub(crate) async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<T> {
        let url = self.build_url(path, query)?;
        let bytes =
            transport::send_with_retries(&self.inner, reqwest::Method::GET, url, Body::None)
                .await?;
        transport::decode_json(&bytes)
    }

    /// Internal: GET `path` with `query`, return raw bytes.
    pub(crate) async fn get_bytes(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<Vec<u8>> {
        let url = self.build_url(path, query)?;
        transport::send_with_retries(&self.inner, reqwest::Method::GET, url, Body::None).await
    }

    /// Internal: POST `path` with JSON `body`, decode response as `T`.
    pub(crate) async fn post_json<B: Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.build_url(path, &[])?;
        let value = serde_json::to_value(body).map_err(Error::Decode)?;
        let bytes = transport::send_with_retries(
            &self.inner,
            reqwest::Method::POST,
            url,
            Body::Json(&value),
        )
        .await?;
        if bytes.is_empty() {
            // Endpoints that legally return 204; T must be deserializable from
            // `null` for this to work (e.g. `Option<X>` or `Value`).
            return transport::decode_json::<T>(b"null");
        }
        transport::decode_json(&bytes)
    }

    /// Internal: PATCH `path` with JSON `body`, decode response as `T`.
    pub(crate) async fn patch_json<B: Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.build_url(path, &[])?;
        let value = serde_json::to_value(body).map_err(Error::Decode)?;
        let bytes = transport::send_with_retries(
            &self.inner,
            reqwest::Method::PATCH,
            url,
            Body::Json(&value),
        )
        .await?;
        if bytes.is_empty() {
            return transport::decode_json::<T>(b"null");
        }
        transport::decode_json(&bytes)
    }

    /// Internal: DELETE `path` and discard the response.
    pub(crate) async fn delete_no_content(&self, path: &str) -> Result<()> {
        let url = self.build_url(path, &[])?;
        transport::send_with_retries(&self.inner, reqwest::Method::DELETE, url, Body::None).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_picks_up_env_var() {
        // SAFETY: tests touch the process env. Run single-threaded if you
        // care; for this assertion we just set and immediately read.
        std::env::set_var("TANGO_API_KEY", "env-key");
        let c = Client::builder().build().expect("build");
        assert_eq!(c.inner.api_key, "env-key");
        std::env::remove_var("TANGO_API_KEY");
    }

    #[test]
    fn explicit_api_key_wins_over_env() {
        std::env::set_var("TANGO_API_KEY", "env-key");
        let c = Client::builder()
            .api_key("explicit-key")
            .build()
            .expect("build");
        assert_eq!(c.inner.api_key, "explicit-key");
        std::env::remove_var("TANGO_API_KEY");
    }

    #[test]
    fn default_base_url() {
        let c = Client::builder().api_key("x").build().expect("build");
        assert_eq!(c.base_url(), crate::shapes::DEFAULT_BASE_URL);
    }

    #[test]
    fn build_url_joins_path_and_query() {
        let c = Client::builder()
            .api_key("x")
            .base_url("https://example.test/".to_string())
            .build()
            .expect("build");
        let url = c
            .build_url(
                "/api/contracts/",
                &[("limit".into(), "25".into()), ("page".into(), "1".into())],
            )
            .expect("url");
        let s = url.to_string();
        assert!(s.starts_with("https://example.test/api/contracts/"));
        assert!(s.contains("limit=25"));
        assert!(s.contains("page=1"));
    }

    #[test]
    fn build_url_handles_missing_leading_slash() {
        let c = Client::builder().api_key("x").build().expect("build");
        let url = c.build_url("api/version/", &[]).expect("url");
        assert!(url.path().ends_with("/api/version/"));
    }

    #[test]
    fn client_is_send_sync_clone() {
        fn assert_send_sync<T: Send + Sync + Clone>() {}
        assert_send_sync::<Client>();
    }
}
