//! Wire tests for resource routes: each method hits the path and query the API documents, and decodes the response shape the API actually serves.

use httpmock::prelude::*;
use serde_json::json;
use std::time::Duration;
use tango::{
    BudgetAccountQuartersOptions, BudgetAccountRecipientsOptions, Client, EntityBudgetFlowsOptions,
    GetDibbsOptions, GetExclusionOptions, GetSbirOptions, ListDibbsAwardsOptions,
    ListDibbsRfpsOptions, ListDibbsRfqsOptions, ListExclusionsOptions,
    ListSbirSolicitationsOptions, ListSbirTopicsOptions,
};

fn make_client(server: &MockServer) -> Client {
    Client::builder()
        .api_key("test-key")
        .base_url(server.base_url())
        .retries(0u32)
        .timeout(Duration::from_secs(5))
        .build()
        .expect("client")
}

fn page(results: serde_json::Value) -> serde_json::Value {
    json!({"count": 1, "next": null, "previous": null, "results": results})
}

#[tokio::test]
async fn get_department_decodes_integer_code() {
    let server = MockServer::start_async().await;
    let m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/departments/97/");
            then.status(200).json_body(
                json!({"abbreviation": "DOD", "code": 97, "name": "Department of Defense"}),
            );
        })
        .await;
    let rec = make_client(&server)
        .get_department("97")
        .await
        .expect("decode");
    m.assert_async().await;
    assert_eq!(
        rec.get("code").and_then(serde_json::Value::as_i64),
        Some(97)
    );
}

#[tokio::test]
async fn get_agency_decodes_string_code_with_integer_department_code() {
    let server = MockServer::start_async().await;
    let _m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/agencies/9700/");
            then.status(200).json_body(json!({
                "abbreviation": "DOD",
                "code": "9700",
                "department": {"code": 97, "name": "Department of Defense"},
                "name": "Department of Defense"
            }));
        })
        .await;
    let agency = make_client(&server)
        .get_agency("9700", None)
        .await
        .expect("decode");
    assert_eq!(agency.code.as_deref(), Some("9700"));
    assert_eq!(
        agency
            .department
            .as_ref()
            .and_then(|d| d.get("code"))
            .and_then(serde_json::Value::as_i64),
        Some(97)
    );
}

#[tokio::test]
async fn dibbs_routes() {
    let server = MockServer::start_async().await;
    let quote_list = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/dibbs/rfqs/")
                .query_param("open", "false")
                .query_param("nsn", "5340-01-123-4567");
            then.status(200).json_body(page(json!([{"uuid": "r1"}])));
        })
        .await;
    let proposal_list = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/dibbs/rfps/")
                .query_param("buyer_code", "PB");
            then.status(200).json_body(page(json!([{"uuid": "p1"}])));
        })
        .await;
    let awards = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/dibbs/awards/")
                .query_param("awardee_cage", "1ABC5");
            then.status(200).json_body(page(json!([{"uuid": "a1"}])));
        })
        .await;
    let get_award = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/dibbs/awards/a1/")
                .query_param("shape", "uuid");
            then.status(200).json_body(json!({"uuid": "a1"}));
        })
        .await;
    let c = make_client(&server);
    let p = c
        .list_dibbs_rfqs(
            ListDibbsRfqsOptions::builder()
                .open(false)
                .nsn("5340-01-123-4567")
                .build(),
        )
        .await
        .expect("rfqs");
    assert_eq!(p.results.len(), 1);
    c.list_dibbs_rfps(ListDibbsRfpsOptions::builder().buyer_code("PB").build())
        .await
        .expect("rfps");
    c.list_dibbs_awards(
        ListDibbsAwardsOptions::builder()
            .awardee_cage("1ABC5")
            .build(),
    )
    .await
    .expect("awards");
    let rec = c
        .get_dibbs_award("a1", Some(GetDibbsOptions::builder().shape("uuid").build()))
        .await
        .expect("award");
    assert_eq!(
        rec.get("uuid").and_then(serde_json::Value::as_str),
        Some("a1")
    );
    quote_list.assert_async().await;
    proposal_list.assert_async().await;
    awards.assert_async().await;
    get_award.assert_async().await;
}

#[tokio::test]
async fn exclusion_routes() {
    let server = MockServer::start_async().await;
    let list = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/exclusions/")
                .query_param("active", "true")
                .query_param("classification_type", "Firm");
            then.status(200)
                .json_body(page(json!([{"exclusion_key": "abc"}])));
        })
        .await;
    let get = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/exclusions/abc/");
            then.status(200).json_body(json!({"exclusion_key": "abc"}));
        })
        .await;
    let c = make_client(&server);
    c.list_exclusions(
        ListExclusionsOptions::builder()
            .active(true)
            .classification_type("Firm")
            .build(),
    )
    .await
    .expect("list");
    c.get_exclusion("abc", None::<GetExclusionOptions>)
        .await
        .expect("get");
    list.assert_async().await;
    get.assert_async().await;
}

#[tokio::test]
async fn sbir_routes() {
    let server = MockServer::start_async().await;
    let topics = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/sbir/topics/")
                .query_param("activity", "open");
            then.status(200)
                .json_body(page(json!([{"topic_id": "t1"}])));
        })
        .await;
    let topic = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/sbir/topics/t1/");
            then.status(200).json_body(json!({"topic_id": "t1"}));
        })
        .await;
    let sols = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/sbir/solicitations/")
                .query_param("out_of_cycle", "false");
            then.status(200)
                .json_body(page(json!([{"solicitation_id": "s1"}])));
        })
        .await;
    let sol = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/sbir/solicitations/s1/");
            then.status(200).json_body(json!({"solicitation_id": "s1"}));
        })
        .await;
    let c = make_client(&server);
    c.list_sbir_topics(ListSbirTopicsOptions::builder().activity("open").build())
        .await
        .expect("topics");
    c.get_sbir_topic("t1", None::<GetSbirOptions>)
        .await
        .expect("topic");
    c.list_sbir_solicitations(
        ListSbirSolicitationsOptions::builder()
            .out_of_cycle(false)
            .build(),
    )
    .await
    .expect("solicitations");
    c.get_sbir_solicitation("s1", None).await.expect("sol");
    topics.assert_async().await;
    topic.assert_async().await;
    sols.assert_async().await;
    sol.assert_async().await;
}

#[tokio::test]
async fn budget_sub_routes_send_their_own_filters() {
    let server = MockServer::start_async().await;
    let quarters = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/budget/accounts/123/quarters/")
                .query_param("tas", "097-2021/2022-0100");
            then.status(200).json_body(json!({
                "count": 1, "next": null, "previous": null,
                "results": [{"tas": "097-2021/2022-0100", "quarter": 1}],
                "federal_account_symbol": "097-0100",
                "fiscal_year": 2022
            }));
        })
        .await;
    let recipients = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/budget/accounts/123/recipients/")
                .query_param("funding_organization_id", "org-uuid");
            then.status(200)
                .json_body(page(json!([{"recipient_id": "UEI1"}])));
        })
        .await;
    let flows = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/entities/UEI1/budget-flows/")
                .query_param("fiscal_year", "2024");
            then.status(200)
                .json_body(page(json!([{"federal_account_symbol": "097-0100"}])));
        })
        .await;
    let c = make_client(&server);
    let q = c
        .get_budget_account_quarters(
            "123",
            Some(
                BudgetAccountQuartersOptions::builder()
                    .tas("097-2021/2022-0100")
                    .build(),
            ),
        )
        .await
        .expect("quarters");
    assert_eq!(q.results.len(), 1);
    c.get_budget_account_recipients(
        "123",
        Some(
            BudgetAccountRecipientsOptions::builder()
                .funding_organization_id("org-uuid")
                .build(),
        ),
    )
    .await
    .expect("recipients");
    c.get_entity_budget_flows(
        "UEI1",
        Some(
            EntityBudgetFlowsOptions::builder()
                .fiscal_year(2024u32)
                .build(),
        ),
    )
    .await
    .expect("flows");
    quarters.assert_async().await;
    recipients.assert_async().await;
    flows.assert_async().await;
}

#[tokio::test]
async fn singleton_detail_routes() {
    let server = MockServer::start_async().await;
    let paths = [
        "/api/contracts/K1/",
        "/api/opportunities/O1/",
        "/api/notices/N1/",
        "/api/forecasts/42/",
        "/api/grants/G1/",
        "/api/subawards/S1/",
        "/api/budget/accounts/123/",
    ];
    let mut mocks = Vec::new();
    for p in paths {
        mocks.push(
            server
                .mock_async(|when, then| {
                    when.method(GET).path(p);
                    then.status(200).json_body(json!({"ok": true}));
                })
                .await,
        );
    }
    let c = make_client(&server);
    c.get_contract("K1", None).await.expect("contract");
    c.get_opportunity("O1", None).await.expect("opportunity");
    c.get_notice("N1", None).await.expect("notice");
    c.get_forecast("42", None).await.expect("forecast");
    c.get_grant("G1", None).await.expect("grant");
    c.get_subaward("S1", None).await.expect("subaward");
    c.get_budget_account("123", None).await.expect("budget");
    for m in &mocks {
        m.assert_async().await;
    }
}

#[tokio::test]
async fn contract_sub_routes() {
    let server = MockServer::start_async().await;
    let subs = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/contracts/K1/subawards/");
            then.status(200).json_body(page(json!([])));
        })
        .await;
    let txns = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/contracts/K1/transactions/");
            then.status(200).json_body(page(json!([])));
        })
        .await;
    let c = make_client(&server);
    c.list_contract_subawards("K1", None).await.expect("subs");
    c.list_contract_transactions("K1", None)
        .await
        .expect("txns");
    subs.assert_async().await;
    txns.assert_async().await;
}
