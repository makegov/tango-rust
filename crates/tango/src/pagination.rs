//! Pagination: [`Page<T>`] envelope + async [`PageStream<T>`].
//!
//! Every list endpoint returns a [`Page<T>`] for a single page of results.
//! For walking every page, resource methods expose an `iterate_*` constructor
//! that returns a [`PageStream<T>`], which implements [`futures::Stream`].
//!
//! ```no_run
//! use futures::TryStreamExt;
//! use tango::Client;
//! # async fn run() -> tango::Result<()> {
//! let client = Client::builder().api_key("…").build()?;
//! let opts = tango::ListContractsOptions::builder()
//!     .awarding_agency("9700")
//!     .build();
//! let mut stream = client.iterate_contracts(opts);
//! while let Some(record) = stream.try_next().await? {
//!     println!("{:?}", record.get("piid"));
//! }
//! # Ok(()) }
//! ```

use crate::client::Client;
use crate::error::{Error, Result};
use futures::stream::Stream;
use serde::Deserialize;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A single page of results from a list endpoint.
///
/// `next`/`previous` are the server-supplied URLs to the adjacent pages;
/// `cursor` is the keyset cursor extracted from `next` for convenience (empty
/// on offset-paginated endpoints).
#[derive(Debug, Clone)]
pub struct Page<T> {
    /// Total number of items across all pages, when the server reports it.
    /// For some endpoints this matches `results.len()`.
    pub count: u64,
    /// URL of the next page, or `None` when on the last page.
    pub next: Option<String>,
    /// URL of the previous page, or `None` when on the first page.
    pub previous: Option<String>,
    /// Opaque page metadata for keyset endpoints.
    pub page_metadata: Option<serde_json::Value>,
    /// Cursor extracted from `next` on cursor-paginated endpoints. Pass it
    /// back via the `cursor` option to fetch the next page. Empty when the
    /// endpoint is offset-paginated or no cursor is set.
    pub cursor: Option<String>,
    /// The actual data for this page.
    pub results: Vec<T>,
}

#[derive(Deserialize)]
struct RawPage<T> {
    #[serde(default)]
    count: Option<serde_json::Value>,
    #[serde(default)]
    next: Option<String>,
    #[serde(default)]
    previous: Option<String>,
    #[serde(default)]
    page_metadata: Option<serde_json::Value>,
    #[serde(default = "Vec::new")]
    results: Vec<T>,
}

impl<T> Page<T> {
    /// Decode a JSON byte slice into a [`Page<T>`].
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self>
    where
        T: for<'de> Deserialize<'de>,
    {
        let raw: RawPage<T> = serde_json::from_slice(bytes).map_err(Error::Decode)?;
        let count = match raw.count {
            Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0),
            Some(serde_json::Value::String(s)) => s.parse::<u64>().unwrap_or(0),
            _ => 0,
        };
        let cursor = raw.next.as_deref().and_then(extract_cursor);
        let mut out = Self {
            count,
            next: raw.next.filter(|s| !s.is_empty()),
            previous: raw.previous.filter(|s| !s.is_empty()),
            page_metadata: raw.page_metadata,
            cursor,
            results: raw.results,
        };
        if out.count == 0 {
            out.count = out.results.len() as u64;
        }
        Ok(out)
    }
}

fn extract_cursor(url: &str) -> Option<String> {
    let parsed = reqwest::Url::parse(url).ok()?;
    parsed
        .query_pairs()
        .find(|(k, _)| k == "cursor")
        .map(|(_, v)| v.into_owned())
        .filter(|v| !v.is_empty())
}

fn extract_page(url: &str) -> Option<u32> {
    let parsed = reqwest::Url::parse(url).ok()?;
    parsed
        .query_pairs()
        .find(|(k, _)| k == "page")
        .and_then(|(_, v)| v.parse::<u32>().ok())
}

/// Future produced by a [`FetchFn`].
type FetchFut<T> = Pin<Box<dyn Future<Output = Result<Page<T>>> + Send>>;

/// Fetcher signature for [`PageStream`]: given the next `(page, cursor)`,
/// return a future for the next [`Page<T>`].
pub(crate) type FetchFn<T> =
    Box<dyn FnMut(Client, Option<u32>, Option<String>) -> FetchFut<T> + Send>;

/// An async stream over every result of a paginated list endpoint.
///
/// Yields `T` items one at a time, fetching successive pages on demand. The
/// stream follows `?cursor=` when the server returns one in `next`, falling
/// back to `?page=` for offset-paginated endpoints.
///
/// Construct with the `iterate_*` methods on [`Client`]; you typically use
/// `try_next().await` (via [`futures::TryStreamExt`]) or `collect_all().await`.
///
/// # Cancellation
///
/// Dropping the stream stops fetching mid-page. In-flight requests are
/// cancelled when their future is dropped (`reqwest`'s standard behavior).
pub struct PageStream<T> {
    client: Client,
    fetch: FetchFn<T>,
    next_page: Option<u32>,
    next_cursor: Option<String>,
    buf: std::vec::IntoIter<T>,
    done: bool,
    in_flight: Option<FetchFut<T>>,
}

// All fields are themselves Unpin (Pin<Box<dyn Future>> is Unpin, IntoIter is
// Unpin, the rest are inert). The Stream impl uses `Pin<&mut Self>` purely as
// a type-system requirement, not because the struct holds self-referential
// state. Marking Unpin lets us call `get_mut` in `poll_next` and freely mutate
// the buffered iterator/page-tracking fields.
impl<T> Unpin for PageStream<T> {}

impl<T> std::fmt::Debug for PageStream<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PageStream")
            .field("next_page", &self.next_page)
            .field("next_cursor", &self.next_cursor)
            .field("done", &self.done)
            .finish_non_exhaustive()
    }
}

impl<T> PageStream<T>
where
    T: Send + 'static,
{
    pub(crate) fn new(client: Client, fetch: FetchFn<T>) -> Self {
        Self {
            client,
            fetch,
            next_page: None,
            next_cursor: None,
            buf: Vec::new().into_iter(),
            done: false,
            in_flight: None,
        }
    }

    /// Drain the stream into a single `Vec<T>`.
    ///
    /// Convenience for callers who want every result. For huge result sets,
    /// prefer iterating with `try_next` to bound memory.
    pub async fn collect_all(self) -> Result<Vec<T>> {
        use futures::TryStreamExt;
        self.try_collect().await
    }
}

impl<T> Stream for PageStream<T>
where
    T: Send + 'static,
{
    type Item = Result<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // PageStream is Unpin (see impl above) — safe to project to &mut Self.
        let this = self.get_mut();
        loop {
            // 1. Drain the buffer FIRST, even after `done` was set. The last
            //    page legitimately holds results after the server says next=null;
            //    yielding them before honoring done is what makes single-page
            //    iteration correct. (Mirrors the F1 Go-iterator bug fix.)
            if let Some(item) = this.buf.next() {
                return Poll::Ready(Some(Ok(item)));
            }
            if this.done {
                return Poll::Ready(None);
            }

            // 2. Drive any in-flight fetch, or start a new one.
            if this.in_flight.is_none() {
                let client = this.client.clone();
                let page = this.next_page;
                let cursor = this.next_cursor.clone();
                let fut = (this.fetch)(client, page, cursor);
                this.in_flight = Some(fut);
            }

            let fut = this
                .in_flight
                .as_mut()
                .expect("we just set in_flight to Some");
            match fut.as_mut().poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(e)) => {
                    this.in_flight = None;
                    this.done = true;
                    return Poll::Ready(Some(Err(e)));
                }
                Poll::Ready(Ok(page)) => {
                    this.in_flight = None;
                    let Page {
                        next,
                        cursor,
                        results,
                        ..
                    } = page;

                    if results.is_empty() {
                        this.done = true;
                        return Poll::Ready(None);
                    }
                    this.buf = results.into_iter();

                    // Decide how to advance to the next page.
                    match next.as_deref() {
                        None => {
                            this.done = true;
                            this.next_page = None;
                            this.next_cursor = None;
                        }
                        Some(next_url) => {
                            if let Some(c) = cursor {
                                this.next_cursor = Some(c);
                                this.next_page = None;
                            } else if let Some(p) = extract_page(next_url) {
                                this.next_page = Some(p);
                                this.next_cursor = None;
                            } else {
                                this.done = true;
                                this.next_page = None;
                                this.next_cursor = None;
                            }
                        }
                    }
                    // Loop back: yield the first buffered item.
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_basic_page() {
        let body = br#"{
            "count": 2,
            "next": null,
            "previous": null,
            "results": [{"a": 1}, {"a": 2}]
        }"#;
        let page: Page<serde_json::Value> = Page::decode(body).expect("decode");
        assert_eq!(page.count, 2);
        assert!(page.next.is_none());
        assert_eq!(page.results.len(), 2);
        assert!(page.cursor.is_none());
    }

    #[test]
    fn decode_string_count() {
        let body = br#"{"count": "42", "results": []}"#;
        let page: Page<serde_json::Value> = Page::decode(body).expect("decode");
        assert_eq!(page.count, 42);
    }

    #[test]
    fn decode_extracts_cursor_from_next() {
        let body = br#"{
            "count": 1,
            "next": "https://tango.example/api/contracts/?cursor=abc123&limit=25",
            "results": [{"piid": "X"}]
        }"#;
        let page: Page<serde_json::Value> = Page::decode(body).expect("decode");
        assert_eq!(page.cursor.as_deref(), Some("abc123"));
        assert!(page.next.is_some());
    }

    #[test]
    fn count_defaults_to_results_len_when_zero() {
        let body = br#"{"results": [{"a": 1}, {"a": 2}, {"a": 3}]}"#;
        let page: Page<serde_json::Value> = Page::decode(body).expect("decode");
        assert_eq!(page.count, 3);
    }

    #[test]
    fn extract_page_parses_query_param() {
        assert_eq!(
            extract_page("https://tango.example/api/contracts/?page=4&limit=25"),
            Some(4)
        );
        assert_eq!(extract_page("https://tango.example/api/contracts/"), None);
    }
}
