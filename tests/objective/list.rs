use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::objective_json;
use sc::{api, commands::objective};

#[tokio::test]
async fn list_objectives() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!([
        objective_json(1, "Q1 Goals", "in progress", "First objective"),
        objective_json(2, "Q2 Goals", "to do", "Second objective"),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::List { archived: false },
    };
    let result = objective::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_objectives_empty() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::List { archived: false },
    };
    let result = objective::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_objectives_filters_archived() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let mut archived_obj = objective_json(2, "Old Goals", "done", "Archived");
    archived_obj["archived"] = serde_json::Value::Bool(true);

    let body = serde_json::json!([
        objective_json(1, "Current Goals", "in progress", "Active"),
        archived_obj,
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::List { archived: false },
    };
    let result = objective::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
