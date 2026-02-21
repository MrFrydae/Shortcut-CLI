use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::DOC_UUID;
use crate::support::epic_json;
use sc::{api, commands::doc};

#[tokio::test]
async fn list_doc_epics() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!([
        epic_json(1, "Epic One", None),
        epic_json(2, "Epic Two", None),
    ]);

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}/epics")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Epics {
            id: DOC_UUID.to_string(),
        },
    };
    let result = doc::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_doc_epics_empty() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}/epics")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Epics {
            id: DOC_UUID.to_string(),
        },
    };
    let result = doc::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
