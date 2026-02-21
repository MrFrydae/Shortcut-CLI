use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{
    default_icon, epic_state_json, epic_workflow_json, full_epic_json, member_json,
};
use crate::{UUID_ALICE, make_create_args};
use sc::{api, commands::epic};

#[tokio::test]
async fn create_epic_minimal() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_epic_json(100, "New Epic", "");

    Mock::given(method("POST"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let create_args = make_create_args("New Epic");
    let args = epic::EpicArgs {
        action: epic::EpicAction::Create(Box::new(create_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_epic_with_owner_mention() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let members_body = serde_json::json!([member_json(
        UUID_ALICE,
        "alice",
        "Alice Smith",
        "admin",
        false,
        Some(default_icon())
    ),]);

    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&members_body))
        .expect(1)
        .mount(&server)
        .await;

    let body = full_epic_json(101, "Owned Epic", "");

    Mock::given(method("POST"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("Owned Epic");
    create_args.owners = vec!["@alice".to_string()];
    let args = epic::EpicArgs {
        action: epic::EpicAction::Create(Box::new(create_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_epic_with_state_name() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let workflow_body = epic_workflow_json(vec![
        epic_state_json(500000010, "To Do", "unstarted", 0),
        epic_state_json(500000011, "In Progress", "started", 1),
        epic_state_json(500000012, "Done", "done", 2),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epic-workflow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&workflow_body))
        .expect(1)
        .mount(&server)
        .await;

    let body = full_epic_json(102, "Stateful Epic", "");

    Mock::given(method("POST"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("Stateful Epic");
    create_args.state = Some("in_progress".to_string());
    let args = epic::EpicArgs {
        action: epic::EpicAction::Create(Box::new(create_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_epic_api_error() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("POST"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(400))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let create_args = make_create_args("Bad Epic");
    let args = epic::EpicArgs {
        action: epic::EpicAction::Create(Box::new(create_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
}
