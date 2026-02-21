use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::DOC_UUID;
use crate::support::doc_json;
use sc::{api, commands::doc};

#[tokio::test]
async fn update_doc_title() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = doc_json(DOC_UUID, Some("New Title"), Some("content"));

    Mock::given(method("PUT"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Update(Box::new(doc::UpdateArgs {
            id: DOC_UUID.to_string(),
            title: Some("New Title".to_string()),
            content: None,
            content_file: None,
            content_format: None,
        })),
    };
    let result = doc::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_doc_content() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = doc_json(DOC_UUID, Some("My Doc"), Some("updated content"));

    Mock::given(method("PUT"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Update(Box::new(doc::UpdateArgs {
            id: DOC_UUID.to_string(),
            title: None,
            content: Some("updated content".to_string()),
            content_file: None,
            content_format: None,
        })),
    };
    let result = doc::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
