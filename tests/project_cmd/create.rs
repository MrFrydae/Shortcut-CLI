use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::project_json;
use sc::{api, commands::project};

#[tokio::test]
async fn create_project_minimal() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = project_json(42, "New Project", None);

    Mock::given(method("POST"))
        .and(path("/api/v3/projects"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Create(Box::new(project::CreateArgs {
            name: "New Project".to_string(),
            team_id: 1,
            description: None,
            color: None,
            abbreviation: None,
            iteration_length: None,
            external_id: None,
        })),
    };
    let result = project::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_project_with_all_fields() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = project_json(43, "Full Project", Some("A fully specified project"));

    Mock::given(method("POST"))
        .and(path("/api/v3/projects"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Create(Box::new(project::CreateArgs {
            name: "Full Project".to_string(),
            team_id: 1,
            description: Some("A fully specified project".to_string()),
            color: Some("#ff0000".to_string()),
            abbreviation: Some("FP".to_string()),
            iteration_length: Some(3),
            external_id: Some("ext-123".to_string()),
        })),
    };
    let result = project::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
