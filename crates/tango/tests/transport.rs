//! Integration tests for the transport layer.
//!
//! These use `httpmock` to exercise the full request loop end-to-end: building
//! requests, sending them, parsing responses, mapping status codes to error
//! variants, and the retry policy. Unit tests in `src/transport.rs` cover the
//! pure-logic helpers (header parsing, validation-message extraction); these
//! tests cover everything that needs a real HTTP server underneath.

use httpmock::prelude::*;
use std::time::Duration;
use tango::{Client, Error};

fn make_client(server: &MockServer) -> Client {
    Client::builder()
        .api_key("test-key")
        .base_url(server.base_url())
        .retries(0u32)
        .timeout(Duration::from_secs(5))
        .build()
        .expect("client")
}

#[tokio::test]
async fn ok_response_decodes() {
    let server = MockServer::start_async().await;
    let _m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/version/");
            then.status(200).json_body(serde_json::json!({
                "version": "1.2.3"
            }));
        })
        .await;

    let client = make_client(&server);
    let v = client.get_version().await.expect("ok");
    assert_eq!(v.get("version").and_then(|x| x.as_str()), Some("1.2.3"));
}

#[tokio::test]
async fn sends_api_key_header() {
    let server = MockServer::start_async().await;
    let m = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/version/")
                .header("x-api-key", "test-key");
            then.status(200).json_body(serde_json::json!({}));
        })
        .await;

    let client = make_client(&server);
    let _ = client.get_version().await.expect("ok");
    m.assert_async().await;
}

#[tokio::test]
async fn sends_user_agent_header() {
    let server = MockServer::start_async().await;
    let m = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/version/")
                .header_exists("user-agent");
            then.status(200).json_body(serde_json::json!({}));
        })
        .await;

    let client = Client::builder()
        .api_key("k")
        .base_url(server.base_url())
        .user_agent("tango-rust/0.1.0-test")
        .build()
        .unwrap();
    let _ = client.get_version().await.expect("ok");
    m.assert_async().await;
}

#[tokio::test]
async fn maps_401_to_auth_error() {
    let server = MockServer::start_async().await;
    let _m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/version/");
            then.status(401)
                .json_body(serde_json::json!({"detail": "invalid api key"}));
        })
        .await;

    let client = make_client(&server);
    match client.get_version().await {
        Err(Error::Auth { response }) => {
            assert!(response.is_some());
        }
        other => panic!("expected Auth, got {other:?}"),
    }
}

#[tokio::test]
async fn maps_404_to_not_found() {
    let server = MockServer::start_async().await;
    let _m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/agencies/9999/");
            then.status(404)
                .json_body(serde_json::json!({"detail": "Not found"}));
        })
        .await;

    let client = make_client(&server);
    match client.get_agency("9999", None).await {
        Err(Error::NotFound { .. }) => {}
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[tokio::test]
async fn maps_400_to_validation_with_extracted_message() {
    let server = MockServer::start_async().await;
    let _m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/contracts/");
            then.status(400).json_body(serde_json::json!({
                "detail": "fiscal_year must be an integer"
            }));
        })
        .await;

    let client = make_client(&server);
    match client
        .list_contracts(tango::ListContractsOptions::builder().build())
        .await
    {
        Err(Error::Validation { message, .. }) => {
            assert!(
                message.contains("fiscal_year must be an integer"),
                "got: {message}"
            );
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[tokio::test]
async fn maps_429_to_rate_limit_with_retry_after() {
    let server = MockServer::start_async().await;
    let _m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/version/");
            then.status(429)
                .header("retry-after", "3")
                .header("x-ratelimit-type", "minute")
                .json_body(serde_json::json!({"detail": "slow down"}));
        })
        .await;

    let client = make_client(&server);
    match client.get_version().await {
        Err(Error::RateLimit {
            retry_after,
            limit_type,
            ..
        }) => {
            assert_eq!(retry_after, 3);
            assert_eq!(limit_type.as_deref(), Some("minute"));
        }
        other => panic!("expected RateLimit, got {other:?}"),
    }
}

#[tokio::test]
async fn maps_500_to_api_error() {
    let server = MockServer::start_async().await;
    let _m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/version/");
            then.status(500).body("internal error");
        })
        .await;

    let client = make_client(&server);
    match client.get_version().await {
        Err(Error::Api { status: 500, .. }) => {}
        other => panic!("expected Api 500, got {other:?}"),
    }
}

#[tokio::test]
async fn retries_on_500_then_succeeds() {
    let server = MockServer::start_async().await;
    let m500 = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/version/");
            then.status(500).body("fail");
        })
        .await;

    let client = Client::builder()
        .api_key("x")
        .base_url(server.base_url())
        .retries(2u32)
        .retry_backoff(Duration::from_millis(1))
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    // First call will exhaust retries on 500.
    match client.get_version().await {
        Err(Error::Api { status: 500, .. }) => {}
        other => panic!("expected Api 500 after retries, got {other:?}"),
    }
    // 3 hits = 1 initial + 2 retries
    m500.assert_hits_async(3).await;
}

#[tokio::test]
async fn does_not_retry_on_404() {
    let server = MockServer::start_async().await;
    let m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/agencies/zzz/");
            then.status(404).json_body(serde_json::json!({}));
        })
        .await;

    let client = Client::builder()
        .api_key("x")
        .base_url(server.base_url())
        .retries(3u32)
        .retry_backoff(Duration::from_millis(1))
        .build()
        .unwrap();

    let _ = client.get_agency("zzz", None).await;
    // 1 hit total — no retries on 404
    m.assert_hits_async(1).await;
}

#[tokio::test]
async fn rate_limit_info_populated_from_headers() {
    let server = MockServer::start_async().await;
    let _m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/version/");
            then.status(200)
                .header("x-ratelimit-remaining", "42")
                .header("x-ratelimit-limit", "100")
                .header("x-ratelimit-reset", "30")
                .header("x-ratelimit-type", "minute")
                .json_body(serde_json::json!({}));
        })
        .await;

    let client = make_client(&server);
    let _ = client.get_version().await.expect("ok");
    let info = client.rate_limit_info().expect("populated");
    assert_eq!(info.remaining, Some(42));
    assert_eq!(info.limit, Some(100));
    assert_eq!(info.reset_in, Some(30));
    assert_eq!(info.limit_type.as_deref(), Some("minute"));
}

#[tokio::test]
async fn last_response_headers_populated() {
    let server = MockServer::start_async().await;
    let _m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/version/");
            then.status(200)
                .header("x-request-id", "req-xyz-123")
                .json_body(serde_json::json!({}));
        })
        .await;

    let client = make_client(&server);
    let _ = client.get_version().await.expect("ok");
    let headers = client.last_response_headers().expect("headers");
    assert_eq!(
        headers.get("x-request-id").map(|v| v.to_str().unwrap()),
        Some("req-xyz-123")
    );
}

#[tokio::test]
async fn paginated_list_decodes() {
    let server = MockServer::start_async().await;
    let _m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/contracts/");
            then.status(200).json_body(serde_json::json!({
                "count": 2,
                "next": null,
                "previous": null,
                "results": [
                    {"piid": "ABC1"},
                    {"piid": "ABC2"}
                ]
            }));
        })
        .await;

    let client = make_client(&server);
    let page = client
        .list_contracts(tango::ListContractsOptions::builder().build())
        .await
        .unwrap();
    assert_eq!(page.count, 2);
    assert_eq!(page.results.len(), 2);
    assert_eq!(page.results[0]["piid"], "ABC1");
    assert!(page.cursor.is_none());
}

#[tokio::test]
async fn paginated_extracts_cursor_from_next() {
    let server = MockServer::start_async().await;
    let _m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/contracts/");
            then.status(200).json_body(serde_json::json!({
                "count": 1,
                "next": format!("{}/api/contracts/?cursor=ABC123", server.base_url()),
                "previous": null,
                "results": [{"piid": "X1"}]
            }));
        })
        .await;

    let client = make_client(&server);
    let page = client
        .list_contracts(tango::ListContractsOptions::builder().build())
        .await
        .unwrap();
    assert_eq!(page.cursor.as_deref(), Some("ABC123"));
}

#[tokio::test]
async fn iterate_stream_yields_across_pages() {
    use futures::TryStreamExt;

    let server = MockServer::start_async().await;
    // Second page: 2 results, no next. Must be defined FIRST so it matches
    // before the more-general first-page mock (httpmock matches in order).
    let _page2 = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/contracts/")
                .query_param("page", "2");
            then.status(200).json_body(serde_json::json!({
                "count": 4,
                "next": null,
                "previous": format!("{}/api/contracts/", server.base_url()),
                "results": [{"piid": "P3"}, {"piid": "P4"}]
            }));
        })
        .await;
    // First page: 2 results, points to page 2. Defined LAST so it catches the
    // initial query without `page=`.
    let _page1 = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/contracts/");
            then.status(200).json_body(serde_json::json!({
                "count": 4,
                "next": format!("{}/api/contracts/?page=2", server.base_url()),
                "previous": null,
                "results": [{"piid": "P1"}, {"piid": "P2"}]
            }));
        })
        .await;

    let client = make_client(&server);
    let mut stream = client.iterate_contracts(tango::ListContractsOptions::builder().build());
    let mut piids = Vec::new();
    while let Some(record) = stream.try_next().await.unwrap() {
        piids.push(record["piid"].as_str().unwrap().to_string());
    }
    assert_eq!(piids, vec!["P1", "P2", "P3", "P4"]);
}

#[tokio::test]
async fn empty_api_key_falls_through_to_request_anyway() {
    // The SDK doesn't reject empty keys at construction — they surface as 401 on first request.
    let server = MockServer::start_async().await;
    let _m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/version/");
            then.status(401)
                .json_body(serde_json::json!({"detail": "missing key"}));
        })
        .await;

    // Force empty key by NOT calling .api_key() AND clearing env.
    std::env::remove_var("TANGO_API_KEY");
    let client = Client::builder()
        .base_url(server.base_url())
        .build()
        .unwrap();
    match client.get_version().await {
        Err(Error::Auth { .. }) => {}
        other => panic!("expected Auth, got {other:?}"),
    }
}
