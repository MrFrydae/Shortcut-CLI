use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::TEMPLATE_UUID;
use crate::support::{entity_template_json_minimal, full_story_json};
use sc::{api, commands::template};

#[tokio::test]
async fn use_template_minimal() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let template_body = entity_template_json_minimal(TEMPLATE_UUID, "Bug Report Template");

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/entity-templates/{TEMPLATE_UUID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&template_body))
        .expect(1)
        .mount(&server)
        .await;

    let story_body = full_story_json(42, "My Bug Report", "");

    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&story_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = template::TemplateArgs {
        action: template::TemplateAction::Use(Box::new(template::UseArgs {
            id: TEMPLATE_UUID.to_string(),
            name: "My Bug Report".to_string(),
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
async fn use_template_with_overrides() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let template_body = entity_template_json_minimal(TEMPLATE_UUID, "Bug Report Template");

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/entity-templates/{TEMPLATE_UUID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&template_body))
        .expect(1)
        .mount(&server)
        .await;

    let story_body = full_story_json(43, "Safari login crash", "Overridden description");

    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&story_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = template::TemplateArgs {
        action: template::TemplateAction::Use(Box::new(template::UseArgs {
            id: TEMPLATE_UUID.to_string(),
            name: "Safari login crash".to_string(),
            description: Some("Overridden description".to_string()),
            description_file: None,
            story_type: Some("bug".to_string()),
            owner: vec![],
            state: None,
            epic_id: Some(100),
            estimate: Some(5),
            labels: vec!["urgent".to_string()],
            iteration_id: None,
            custom_fields: vec![],
        })),
    };
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}
