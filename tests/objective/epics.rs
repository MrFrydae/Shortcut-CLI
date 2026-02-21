use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::epic_json;
use sc::{api, commands::objective};

#[tokio::test]
async fn list_objective_epics() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!([
        epic_json(1, "Epic One", None),
        epic_json(2, "Epic Two", None),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives/42/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Epics {
            id: 42,
            desc: false,
        },
    };
    let result = objective::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_objective_epics_empty() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives/42/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Epics {
            id: 42,
            desc: false,
        },
    };
    let result = objective::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
