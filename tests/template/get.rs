use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::TEMPLATE_UUID;
use crate::support::entity_template_json;
use sc::{api, commands::template};

#[tokio::test]
async fn get_template() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = entity_template_json(TEMPLATE_UUID, "Bug Report Template");

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/entity-templates/{TEMPLATE_UUID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = template::TemplateArgs {
        action: template::TemplateAction::Get {
            id: TEMPLATE_UUID.to_string(),
        },
    };
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_template_not_found() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/entity-templates/{TEMPLATE_UUID}")))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = template::TemplateArgs {
        action: template::TemplateAction::Get {
            id: TEMPLATE_UUID.to_string(),
        },
    };
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());
}
