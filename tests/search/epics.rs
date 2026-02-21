use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::make_query;
use crate::support::search_epic_result_json;
use sc::{api, commands::search};

#[tokio::test]
async fn search_epics_with_results() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [search_epic_result_json(10, "Auth Epic")],
        "next": null,
        "total": 1,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Epics(make_query("auth")),
    };
    let result = search::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn search_epics_empty() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [],
        "next": null,
        "total": 0,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Epics(make_query("nothing")),
    };
    let result = search::run(&args, &client).await;
    assert!(result.is_ok());
}
