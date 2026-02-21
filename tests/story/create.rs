use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{
    default_icon, full_story_json, make_dry_run_output, member_json, workflow_json,
    workflow_state_json,
};
use crate::{UUID_ALICE, UUID_BOB, make_create_args};
use sc::{api, commands::story};

#[tokio::test]
async fn create_story_minimal() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_story_json(123, "Fix login bug", "");

    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let create_args = make_create_args("Fix login bug");
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_story_with_owner_mention() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let members_body = serde_json::json!([
        member_json(
            UUID_ALICE,
            "alice",
            "Alice Smith",
            "admin",
            false,
            Some(default_icon())
        ),
        member_json(UUID_BOB, "bob", "Bob Jones", "member", false, None),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&members_body))
        .expect(1)
        .mount(&server)
        .await;

    let body = full_story_json(124, "My Story", "");

    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("My Story");
    create_args.owner = vec!["@alice".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_story_with_state_name() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let workflows_body = serde_json::json!([workflow_json(
        500000006,
        "Default",
        vec![
            workflow_state_json(500000007, "Unstarted", "unstarted", 0),
            workflow_state_json(500000008, "In Progress", "started", 1),
            workflow_state_json(500000009, "Done", "done", 2),
        ]
    ),]);

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&workflows_body))
        .expect(1)
        .mount(&server)
        .await;

    let body = full_story_json(125, "Stateful Story", "");

    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("Stateful Story");
    create_args.state = Some("in_progress".to_string());
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn dry_run_story_create_shows_request() {
    let (out, buf) = make_dry_run_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // No POST mock — if a POST is sent, wiremock returns 404 and the test fails

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("Test Story");
    create_args.story_type = Some("bug".to_string());
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(output.contains("[dry-run] POST /api/v3/stories"));
    assert!(output.contains("\"name\": \"Test Story\""));
    assert!(output.contains("\"story_type\": \"bug\""));
}

#[tokio::test]
async fn dry_run_story_create_with_owner_resolves_members() {
    let (out, buf) = make_dry_run_output();
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

    // No POST mock — dry-run should not send one

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("Owner Story");
    create_args.owner = vec!["@alice".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(output.contains("[dry-run] POST /api/v3/stories"));
    // The resolved UUID should appear in the output
    assert!(output.contains(UUID_ALICE));
}
