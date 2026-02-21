use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::objective_json;
use sc::{api, commands::objective};

#[tokio::test]
async fn delete_objective_with_confirm() {
    let server = MockServer::start().await;

    let body = objective_json(42, "To Delete", "to do", "");

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/objectives/42"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Delete {
            id: 42,
            confirm: true,
        },
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_objective_without_confirm_errors() {
    let server = MockServer::start().await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Delete {
            id: 42,
            confirm: false,
        },
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}
