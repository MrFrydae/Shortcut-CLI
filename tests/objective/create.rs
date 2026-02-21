use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::objective_json;
use sc::{api, commands::objective};

#[tokio::test]
async fn create_objective_minimal() {
    let server = MockServer::start().await;

    let body = objective_json(42, "New Objective", "to do", "");

    Mock::given(method("POST"))
        .and(path("/api/v3/objectives"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Create(Box::new(objective::CreateArgs {
            name: "New Objective".to_string(),
            description: None,
            state: None,
            categories: vec![],
        })),
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_objective_with_all_fields() {
    let server = MockServer::start().await;

    let body = objective_json(43, "Full Objective", "in progress", "A description");

    Mock::given(method("POST"))
        .and(path("/api/v3/objectives"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Create(Box::new(objective::CreateArgs {
            name: "Full Objective".to_string(),
            description: Some("A description".to_string()),
            state: Some("in progress".to_string()),
            categories: vec!["Engineering".to_string()],
        })),
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}
