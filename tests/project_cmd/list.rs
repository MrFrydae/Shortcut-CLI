use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::project_json;
use sc::{api, commands::project};

#[tokio::test]
async fn list_projects() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!([
        project_json(1, "Backend", Some("Backend services")),
        project_json(2, "Frontend", Some("Frontend apps")),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/projects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::List { archived: false },
    };
    let result = project::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_projects_empty() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/projects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::List { archived: false },
    };
    let result = project::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_projects_filters_archived() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let mut archived_proj = project_json(2, "Old Project", None);
    archived_proj["archived"] = serde_json::Value::Bool(true);

    let body = serde_json::json!([
        project_json(1, "Active Project", Some("Still active")),
        archived_proj,
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/projects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::List { archived: false },
    };
    let result = project::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
