use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::make_query;
use crate::support::{search_epic_result_json, search_story_result_json};
use sc::{api, commands::search};

#[tokio::test]
async fn search_all_with_results() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "stories": {
            "data": [search_story_result_json(1, "Login bug", "bug")],
            "next": null,
            "total": 1,
        },
        "epics": {
            "data": [search_epic_result_json(10, "Authentication")],
            "next": null,
            "total": 1,
        },
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::All(make_query("login")),
    };
    let result = search::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn search_all_empty() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!({});

    Mock::given(method("GET"))
        .and(path("/api/v3/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::All(make_query("nonexistent")),
    };
    let result = search::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
