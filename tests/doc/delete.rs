use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::DOC_UUID;
use crate::support::doc_json;
use sc::{api, commands::doc};

#[tokio::test]
async fn delete_doc_with_confirm() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let get_body = doc_json(DOC_UUID, Some("To Delete"), Some("content"));

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Delete {
            id: DOC_UUID.to_string(),
            confirm: true,
        },
    };
    let result = doc::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_doc_without_confirm_errors() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Delete {
            id: DOC_UUID.to_string(),
            confirm: false,
        },
    };
    let result = doc::run(&args, &client, &out).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}
