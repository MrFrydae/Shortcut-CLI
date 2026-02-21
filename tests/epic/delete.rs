use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::full_epic_json;
use sc::{api, commands::epic};

#[tokio::test]
async fn delete_epic_with_confirm() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_epic_json(42, "My Epic", "desc");

    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic::EpicArgs {
        action: epic::EpicAction::Delete {
            id: 42,
            confirm: true,
        },
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_epic_without_confirm_errors() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic::EpicArgs {
        action: epic::EpicAction::Delete {
            id: 42,
            confirm: false,
        },
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}

#[tokio::test]
async fn delete_epic_not_found() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/epics/999"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic::EpicArgs {
        action: epic::EpicAction::Delete {
            id: 999,
            confirm: true,
        },
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
}
