use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::make_list_args;
use crate::support::epic_json;
use sc::{api, commands::epic};

#[tokio::test]
async fn list_epics_prints_names() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

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
    let args = make_list_args(false);
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epics_with_descriptions() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = serde_json::json!([epic_json(1, "Epic One", Some("Description of epic one")),]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .and(query_param("includes_description", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_list_args(true);
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epics_empty() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = serde_json::json!([]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_list_args(false);
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epics_api_error() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(400))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_list_args(false);
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
}
