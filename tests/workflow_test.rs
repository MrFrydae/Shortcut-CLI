mod support;

use sc::{api, commands::workflow};
use support::{workflow_json, workflow_state_json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_args(list: bool, id: Option<i64>) -> workflow::WorkflowArgs {
    workflow::WorkflowArgs { list, id }
}

#[tokio::test]
async fn list_workflows_prints_names() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!([
        workflow_json(1, "Engineering Workflow", vec![]),
        workflow_json(2, "Design Workflow", vec![]),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(true, None);
    let result = workflow::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_workflow_by_id() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let states = vec![
        workflow_state_json(100, "Backlog", "Unstarted", 0),
        workflow_state_json(101, "In Progress", "Started", 1),
        workflow_state_json(102, "Done", "Finished", 2),
    ];
    let body = workflow_json(1, "Engineering Workflow", states);

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(false, Some(1));
    let result = workflow::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_workflows_empty() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!([]);

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(true, None);
    let result = workflow::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_workflows_api_error() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(400))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(true, None);
    let result = workflow::run(&args, &client, &out).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn no_flags_does_nothing() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    // No mocks registered — any HTTP call will cause a panic via expect(0).

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(false, None);
    let result = workflow::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
