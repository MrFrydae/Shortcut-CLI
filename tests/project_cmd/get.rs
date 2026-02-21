use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::project_json;
use sc::{api, commands::project};

#[tokio::test]
async fn get_project() {
    let server = MockServer::start().await;

    let body = project_json(42, "My Project", Some("A great project"));

    Mock::given(method("GET"))
        .and(path("/api/v3/projects/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Get { id: 42 },
    };
    let result = project::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_project_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/projects/999"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Get { id: 999 },
    };
    let result = project::run(&args, &client).await;
    assert!(result.is_err());
}
