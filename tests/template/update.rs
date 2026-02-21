use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::TEMPLATE_UUID;
use crate::support::entity_template_json;
use sc::{api, commands::template};

#[tokio::test]
async fn update_template_name() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = entity_template_json(TEMPLATE_UUID, "Renamed Template");

    Mock::given(method("PUT"))
        .and(path(format!("/api/v3/entity-templates/{TEMPLATE_UUID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = template::TemplateArgs {
        action: template::TemplateAction::Update(Box::new(template::UpdateArgs {
            id: TEMPLATE_UUID.to_string(),
            name: Some("Renamed Template".to_string()),
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
async fn update_template_story_contents() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = entity_template_json(TEMPLATE_UUID, "Bug Report Template");

    Mock::given(method("PUT"))
        .and(path(format!("/api/v3/entity-templates/{TEMPLATE_UUID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = template::TemplateArgs {
        action: template::TemplateAction::Update(Box::new(template::UpdateArgs {
            id: TEMPLATE_UUID.to_string(),
            name: None,
            story_name: None,
            description: Some("Updated description".to_string()),
            description_file: None,
            story_type: Some("bug".to_string()),
            owner: vec![],
            state: None,
            epic_id: None,
            estimate: Some(3),
            labels: vec![],
            iteration_id: None,
            custom_fields: vec![],
        })),
    };
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}
