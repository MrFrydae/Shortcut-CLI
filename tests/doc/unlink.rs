use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::DOC_UUID;
use sc::{api, commands::doc};

#[tokio::test]
async fn unlink_doc_from_epic() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}/epics/42")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Unlink {
            doc_id: DOC_UUID.to_string(),
            epic_id: 42,
        },
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}
