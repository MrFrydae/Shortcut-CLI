use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{
    custom_field_enum_value_json, custom_field_json, default_icon, full_story_json,
    make_dry_run_output, make_output, member_json, mount_default_workflow, story_custom_field_json,
    workflow_json, workflow_state_json,
};
use crate::{UUID_ALICE, UUID_BOB, UUID_FIELD_1, UUID_VAL_A, make_create_args};
use shortcut_cli::output::{ColorMode, OutputConfig, OutputMode};
use shortcut_cli::{api, commands::story};

#[tokio::test]
async fn create_story_minimal() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    mount_default_workflow(&server).await;

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
async fn create_story_json_emits_full_story() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Json, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    mount_default_workflow(&server).await;

    // Resolves "Priority=High" before the write and `field_name` after it.
    let cf_body = serde_json::json!([custom_field_json(
        UUID_FIELD_1,
        "Priority",
        vec![custom_field_enum_value_json(UUID_VAL_A, "High", 0, true)]
    )]);
    Mock::given(method("GET"))
        .and(path("/api/v3/custom-fields"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&cf_body))
        .expect(1)
        .mount(&server)
        .await;

    let group_id = "cccccccc-cccc-cccc-cccc-cccccccccccc";
    let mut body = full_story_json(123, "Fix login bug", "");
    body["group_id"] = serde_json::json!(group_id);
    body["iteration_id"] = serde_json::json!(88);
    body["custom_fields"] =
        serde_json::json!([story_custom_field_json(UUID_FIELD_1, UUID_VAL_A, "High")]);

    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("Fix login bug");
    create_args.custom_fields = vec!["Priority=High".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "{result:?}");

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(json["id"], 123);
    assert_eq!(json["name"], "Fix login bug");
    assert_eq!(json["group_id"], group_id);
    assert_eq!(json["iteration_id"], 88);
    assert_eq!(json["app_url"], "https://app.shortcut.com/test/story/123");
    assert_eq!(json["workflow_state_id"], 500000007);
    assert_eq!(json["custom_fields"][0]["field_id"], UUID_FIELD_1);
    assert_eq!(json["custom_fields"][0]["value_id"], UUID_VAL_A);
    assert_eq!(json["custom_fields"][0]["value"], "High");
    assert_eq!(json["custom_fields"][0]["field_name"], "Priority");
}

#[tokio::test]
async fn create_story_with_owner_mention() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    mount_default_workflow(&server).await;

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

    mount_default_workflow(&server).await;

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

    mount_default_workflow(&server).await;

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

#[tokio::test]
async fn create_story_with_parent_story_id() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    mount_default_workflow(&server).await;

    let body = full_story_json(130, "Child Story", "");

    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("Child Story");
    create_args.parent_story_id = Some(42);
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn dry_run_story_create_with_parent_story_id() {
    let (out, buf) = make_dry_run_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    mount_default_workflow(&server).await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("Child Story");
    create_args.parent_story_id = Some(123);
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(output.contains("[dry-run] POST /api/v3/stories"));
    assert!(output.contains("\"parent_story_id\": 123"));
}
