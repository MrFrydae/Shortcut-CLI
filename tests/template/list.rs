use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::entity_template_json;
use crate::{TEMPLATE_UUID, TEMPLATE_UUID2};
use sc::{api, commands::template};

#[tokio::test]
async fn list_templates() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = serde_json::json!([
        entity_template_json(TEMPLATE_UUID, "Bug Report Template"),
        entity_template_json(TEMPLATE_UUID2, "Feature Request"),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/entity-templates"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = template::TemplateArgs {
        action: template::TemplateAction::List,
    };
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_templates_empty() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/entity-templates"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = template::TemplateArgs {
        action: template::TemplateAction::List,
    };
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}
