use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::doc_slim_json;
use crate::{DOC_UUID, DOC_UUID2};
use sc::{api, commands::doc};

#[tokio::test]
async fn list_docs() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        doc_slim_json(DOC_UUID, Some("Design Spec")),
        doc_slim_json(DOC_UUID2, Some("RFC: New API")),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/documents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::List,
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_docs_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/documents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::List,
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}
