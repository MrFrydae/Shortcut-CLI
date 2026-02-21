use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::TEMPLATE_UUID;
use crate::support::entity_template_json;
use sc::{api, commands::template};

#[tokio::test]
async fn create_template_minimal() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = entity_template_json(TEMPLATE_UUID, "New Template");

    Mock::given(method("POST"))
        .and(path("/api/v3/entity-templates"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = template::TemplateArgs {
        action: template::TemplateAction::Create(Box::new(template::CreateArgs {
            name: "New Template".to_string(),
            story_name: None,
            description: None,
            description_file: None,
            story_type: None,
            owner: vec![],
            state: None,
            epic_id: None,
            estimate: None,
            labels: vec![],
            iteration_id: None,
            custom_fields: vec![],
        })),
    };
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_template_with_story_contents() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = entity_template_json(TEMPLATE_UUID, "Bug Report");

    Mock::given(method("POST"))
        .and(path("/api/v3/entity-templates"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = template::TemplateArgs {
        action: template::TemplateAction::Create(Box::new(template::CreateArgs {
            name: "Bug Report".to_string(),
            story_name: Some("Bug: ".to_string()),
            description: Some("## Steps to Reproduce\n\n## Expected\n\n## Actual".to_string()),
            description_file: None,
            story_type: Some("bug".to_string()),
            owner: vec![],
            state: None,
            epic_id: None,
            estimate: Some(1),
            labels: vec!["bug".to_string()],
            iteration_id: None,
            custom_fields: vec![],
        })),
    };
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}
