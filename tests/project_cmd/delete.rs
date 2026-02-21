use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::project_json;
use sc::{api, commands::project};

#[tokio::test]
async fn delete_project_with_confirm() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = project_json(42, "To Delete", None);

    Mock::given(method("GET"))
        .and(path("/api/v3/projects/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/projects/42"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Delete {
            id: 42,
            confirm: true,
        },
    };
    let result = project::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_project_without_confirm_errors() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Delete {
            id: 42,
            confirm: false,
        },
    };
    let result = project::run(&args, &client, &out).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}
