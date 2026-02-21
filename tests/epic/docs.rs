use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::doc_slim_json;
use sc::{api, commands::epic};

#[tokio::test]
async fn list_epic_documents() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = serde_json::json!([
        doc_slim_json("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee", Some("Design Spec")),
        doc_slim_json("11111111-2222-3333-4444-555555555555", Some("RFC Doc")),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42/documents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic::EpicArgs {
        action: epic::EpicAction::Docs { id: 42 },
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epic_documents_empty() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42/documents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic::EpicArgs {
        action: epic::EpicAction::Docs { id: 42 },
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}
