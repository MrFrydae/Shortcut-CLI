use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::DOC_UUID;
use crate::support::doc_json;
use sc::{api, commands::doc};

#[tokio::test]
async fn get_doc() {
    let server = MockServer::start().await;

    let body = doc_json(DOC_UUID, Some("My Document"), Some("# Hello\nWorld"));

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Get {
            id: DOC_UUID.to_string(),
        },
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_doc_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}")))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Get {
            id: DOC_UUID.to_string(),
        },
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_err());
}
