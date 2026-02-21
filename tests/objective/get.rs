use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{epic_json, objective_json};
use sc::{api, commands::objective};

#[tokio::test]
async fn get_objective() {
    let server = MockServer::start().await;

    let body = objective_json(42, "My Objective", "in progress", "Details here");

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    // Also mock the epics endpoint that get calls
    Mock::given(method("GET"))
        .and(path("/api/v3/objectives/42/epics"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([epic_json(
                1,
                "Related Epic",
                None
            ),])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Get { id: 42 },
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_objective_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives/999"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Get { id: 999 },
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_err());
}
