use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::make_query;
use crate::support::search_story_result_json;
use sc::{api, commands::search};

#[tokio::test]
async fn search_stories_with_results() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [
            search_story_result_json(1, "Login bug", "bug"),
            search_story_result_json(2, "Signup feature", "feature"),
        ],
        "next": null,
        "total": 2,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/stories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Stories(make_query("login")),
    };
    let result = search::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn search_stories_empty() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [],
        "next": null,
        "total": 0,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/stories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Stories(make_query("nothing")),
    };
    let result = search::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn search_stories_with_pagination() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [search_story_result_json(1, "Story One", "feature")],
        "next": "cursor-abc123",
        "total": 50,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/stories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Stories(make_query("story")),
    };
    let result = search::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
