use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{epic_state_json, epic_workflow_json, full_epic_json};
use sc::{api, commands::epic};

#[tokio::test]
async fn get_epic_prints_details() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_epic_json(42, "My Epic", "Some description");

    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let workflow_body = epic_workflow_json(vec![epic_state_json(1, "To Do", "unstarted", 0)]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epic-workflow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&workflow_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic::EpicArgs {
        action: epic::EpicAction::Get { id: 42 },
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_epic_not_found() {
    let out = crate::support::make_output();
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
        action: epic::EpicAction::Get { id: 999 },
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn get_epic_with_cached_state() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Pre-populate epic state cache
    let cache: std::collections::HashMap<String, i64> = [("to do".to_string(), 1)].into();
    let cache_path = tmp.path().join("epic_state_cache.json");
    std::fs::write(&cache_path, serde_json::to_string(&cache).unwrap()).unwrap();

    let body = full_epic_json(42, "My Epic", "desc");

    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    // No epic-workflow mock — cache should be used for state name lookup
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic::EpicArgs {
        action: epic::EpicAction::Get { id: 42 },
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}
