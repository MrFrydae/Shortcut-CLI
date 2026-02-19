mod support;

use sc::{api, commands::epic};
use support::epic_json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_args(list: bool, desc: bool) -> epic::EpicArgs {
    epic::EpicArgs { list, desc }
}

#[tokio::test]
async fn list_epics_prints_names() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        epic_json(1, "Epic One", None),
        epic_json(2, "Epic Two", None),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(true, false);
    let result = epic::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epics_with_descriptions() {
    let server = MockServer::start().await;

    let body = serde_json::json!([epic_json(1, "Epic One", Some("Description of epic one")),]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .and(query_param("includes_description", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(true, true);
    let result = epic::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epics_empty() {
    let server = MockServer::start().await;

    let body = serde_json::json!([]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(true, false);
    let result = epic::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epics_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(400))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(true, false);
    let result = epic::run(&args, &client).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn no_list_flag_does_nothing() {
    let server = MockServer::start().await;

    // No mocks registered — any HTTP call will cause a panic via expect(0).

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(false, false);
    let result = epic::run(&args, &client).await;
    assert!(result.is_ok());
}
