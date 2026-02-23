use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use shortcut_cli::output::{ColorMode, OutputConfig, OutputMode};

use crate::support::{
    default_icon, full_story_json, make_dry_run_output, member_json, workflow_json,
    workflow_state_json,
};
use crate::{UUID_ALICE, make_update_args};
use shortcut_cli::{api, commands::story};

/// Build a full story JSON with a specific workflow_state_id.
fn full_story_json_with_state(id: i64, name: &str, desc: &str, state_id: i64) -> serde_json::Value {
    let mut story = full_story_json(id, name, desc);
    story["workflow_state_id"] = serde_json::json!(state_id);
    story
}

#[tokio::test]
async fn update_story_name_and_description() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_story_json(42, "New Title", "New description");

    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.name = Some("New Title".to_string());
    update_args.description = Some("New description".to_string());
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_story_state() {
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

    let body = full_story_json(42, "My Story", "desc");

    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.state = Some("Done".to_string());
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_story_owner() {
    let out = crate::support::make_output();
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

    let body = full_story_json(42, "My Story", "desc");

    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.owner = vec!["@alice".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

// --- State resolution tests ---

#[tokio::test]
async fn state_resolution_numeric_passes_through() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_story_json(42, "My Story", "desc");

    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    // No workflows mock — numeric ID should not trigger API call
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.state = Some("500000008".to_string());
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn state_resolution_cache_hit() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Pre-populate cache
    let cache: std::collections::HashMap<String, i64> =
        [("in progress".to_string(), 500000008)].into();
    let cache_path = tmp.path().join("workflow_state_cache.json");
    std::fs::write(&cache_path, serde_json::to_string(&cache).unwrap()).unwrap();

    let body = full_story_json(42, "My Story", "desc");

    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    // No workflows mock — cache hit should avoid API call
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.state = Some("in_progress".to_string());
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn state_ambiguous_error() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Two workflows with the same state name
    let workflows_body = serde_json::json!([
        workflow_json(
            1,
            "Workflow A",
            vec![workflow_state_json(100, "In Progress", "started", 0),]
        ),
        workflow_json(
            2,
            "Workflow B",
            vec![workflow_state_json(200, "In Progress", "started", 0),]
        ),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&workflows_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.state = Some("In Progress".to_string());
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Ambiguous"));
}

#[tokio::test]
async fn unknown_state_name_errors() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let workflows_body = serde_json::json!([workflow_json(
        500000006,
        "Default",
        vec![
            workflow_state_json(500000007, "Unstarted", "unstarted", 0),
            workflow_state_json(500000008, "In Progress", "started", 1),
        ]
    ),]);

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&workflows_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.state = Some("nonexistent".to_string());
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Unknown workflow state"));
    assert!(err.contains("Unstarted"));
    assert!(err.contains("In Progress"));
}

#[tokio::test]
async fn dry_run_story_update_shows_request() {
    let (out, buf) = make_dry_run_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // No PUT mock — dry-run should not send one

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.name = Some("Updated Name".to_string());
    update_args.estimate = Some(5);
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(output.contains("[dry-run] PUT /api/v3/stories/42"));
    assert!(output.contains("\"name\": \"Updated Name\""));
    assert!(output.contains("\"estimate\": 5"));
    // Fields not set should NOT appear
    assert!(!output.contains("\"description\""));
}

// --- --unless-state tests ---

fn standard_workflows_body() -> serde_json::Value {
    serde_json::json!([workflow_json(
        500000006,
        "Default",
        vec![
            workflow_state_json(500000007, "Unstarted", "unstarted", 0),
            workflow_state_json(500000008, "In Progress", "started", 1),
            workflow_state_json(500000009, "Done", "done", 2),
            workflow_state_json(500000010, "In Review", "started", 3),
        ]
    )])
}

#[tokio::test]
async fn unless_state_skips_when_matched() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Story is currently in "Done" (state_id 500000009)
    let get_body = full_story_json_with_state(42, "My Story", "desc", 500000009);
    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(standard_workflows_body()))
        .mount(&server)
        .await;

    // PUT should NOT be called
    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.state = Some("In Progress".to_string());
    update_args.unless_state = vec!["Done".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn unless_state_proceeds_when_not_matched() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Story is currently in "Unstarted" (state_id 500000007)
    let get_body = full_story_json_with_state(42, "My Story", "desc", 500000007);
    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(standard_workflows_body()))
        .mount(&server)
        .await;

    let put_body = full_story_json(42, "My Story", "desc");
    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&put_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.state = Some("In Progress".to_string());
    update_args.unless_state = vec!["Done".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn unless_state_multiple_values() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Story is currently in "In Review" (state_id 500000010)
    let get_body = full_story_json_with_state(42, "My Story", "desc", 500000010);
    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(standard_workflows_body()))
        .mount(&server)
        .await;

    // PUT should NOT be called
    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.state = Some("In Progress".to_string());
    update_args.unless_state = vec!["In Review".to_string(), "Done".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn unless_state_normalizes_names() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Story is currently in "In Progress" (state_id 500000008)
    let get_body = full_story_json_with_state(42, "My Story", "desc", 500000008);
    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(standard_workflows_body()))
        .mount(&server)
        .await;

    // PUT should NOT be called — "in_progress" normalizes to match "In Progress"
    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.state = Some("Done".to_string());
    update_args.unless_state = vec!["in_progress".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn unless_state_json_output() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Json, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Story is currently in "Done" (state_id 500000009)
    let get_body = full_story_json_with_state(42, "My Story", "desc", 500000009);
    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(standard_workflows_body()))
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.state = Some("In Progress".to_string());
    update_args.unless_state = vec!["Done".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(json["id"], 42);
    assert_eq!(json["skipped"], true);
    assert_eq!(json["current_state"], "Done");
    assert!(json["reason"].as_str().unwrap().contains("--unless-state"));
}

// --- --add-owner tests ---

#[tokio::test]
async fn add_owner_appends_to_existing() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Story currently has alice as owner
    let mut get_body = full_story_json(42, "My Story", "desc");
    get_body["owner_ids"] = serde_json::json!([UUID_ALICE]);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_body))
        .expect(1)
        .mount(&server)
        .await;

    // Members API for resolving @bob
    let members_body = serde_json::json!([
        member_json(
            UUID_ALICE,
            "alice",
            "Alice Smith",
            "admin",
            false,
            Some(default_icon())
        ),
        member_json(
            crate::UUID_BOB,
            "bob",
            "Bob Jones",
            "member",
            false,
            Some(default_icon())
        ),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&members_body))
        .expect(1)
        .mount(&server)
        .await;

    let put_body = full_story_json(42, "My Story", "desc");

    let put_mock = Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&put_body))
        .expect(1)
        .mount_as_scoped(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.add_owner = vec!["@bob".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());

    // Verify the PUT request contained both alice and bob
    let requests = put_mock.received_requests().await;
    assert_eq!(requests.len(), 1);
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    let sent_ids = body["owner_ids"].as_array().unwrap();
    assert_eq!(sent_ids.len(), 2);
    assert!(sent_ids.contains(&serde_json::json!(UUID_ALICE)));
    assert!(sent_ids.contains(&serde_json::json!(crate::UUID_BOB)));
}
