use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::TEMPLATE_UUID;
use crate::support::entity_template_json;
use sc::{api, commands::template};

#[tokio::test]
async fn delete_template_with_confirm() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let get_body = entity_template_json(TEMPLATE_UUID, "To Delete");

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/entity-templates/{TEMPLATE_UUID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path(format!("/api/v3/entity-templates/{TEMPLATE_UUID}")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = template::TemplateArgs {
        action: template::TemplateAction::Delete {
            id: TEMPLATE_UUID.to_string(),
            confirm: true,
        },
    };
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_template_without_confirm() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = template::TemplateArgs {
        action: template::TemplateAction::Delete {
            id: TEMPLATE_UUID.to_string(),
            confirm: false,
        },
    };
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}
