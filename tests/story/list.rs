use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{
    default_icon, label_json, member_json, story_json, workflow_json, workflow_state_json,
};
use crate::{UUID_ALICE, make_list_args};
use shortcut_cli::output::{ColorMode, OutputConfig, OutputMode};
use shortcut_cli::{api, commands::story};

#[tokio::test]
async fn list_stories_json_emits_full_stories() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Json, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let group_id = "cccccccc-cccc-cccc-cccc-cccccccccccc";
    let mut first = story_json(1, "Story One", None);
    first["group_id"] = serde_json::json!(group_id);
    first["labels"] = serde_json::json!([label_json(1, "bug")]);
    let body = serde_json::json!([first, story_json(2, "Story Two", None)]);

    Mock::given(method("POST"))
        .and(path("/api/v3/stories/search"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;
    // No /workflows or /custom-fields mock: JSON mode must not need either.

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::List(Box::new(make_list_args())),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "{result:?}");

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    let items = json.as_array().unwrap();
    assert_eq!(items.len(), 2);
    // Existing projection fields keep their names and shapes.
    assert_eq!(items[0]["id"], 1);
    assert_eq!(items[0]["name"], "Story One");
    assert_eq!(items[0]["story_type"], "feature");
    assert_eq!(items[0]["workflow_state_id"], 500000007);
    // Full object fields are now present.
    assert_eq!(items[0]["group_id"], group_id);
    assert_eq!(items[0]["app_url"], "https://app.shortcut.com/test/story/1");
    assert_eq!(items[0]["labels"], serde_json::json!(["bug"]));
    assert_eq!(items[0]["label_details"][0]["name"], "bug");
    assert!(items[1]["group_id"].is_null());
    assert_eq!(items[1]["custom_fields"], serde_json::json!([]));
}

#[tokio::test]
async fn list_stories_minimal() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = serde_json::json!([
        story_json(1, "Story One", None),
        story_json(2, "Story Two", None),
    ]);

    Mock::given(method("POST"))
        .and(path("/api/v3/stories/search"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let workflows_body = serde_json::json!([workflow_json(
        500000006,
        "Default",
        vec![
            workflow_state_json(500000007, "Unstarted", "unstarted", 0),
            workflow_state_json(500000008, "In Progress", "started", 1),
            workflow_state_json(500000009, "Done", "done", 2),
        ]
    )]);

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&workflows_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let list_args = make_list_args();
    let args = story::StoryArgs {
        action: story::StoryAction::List(Box::new(list_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_stories_empty() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("POST"))
        .and(path("/api/v3/stories/search"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let list_args = make_list_args();
    let args = story::StoryArgs {
        action: story::StoryAction::List(Box::new(list_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_stories_with_owner_filter() {
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

    let body = serde_json::json!([story_json(1, "Alice's Story", None)]);

    Mock::given(method("POST"))
        .and(path("/api/v3/stories/search"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let workflows_body = serde_json::json!([workflow_json(
        500000006,
        "Default",
        vec![
            workflow_state_json(500000007, "Unstarted", "unstarted", 0),
            workflow_state_json(500000008, "In Progress", "started", 1),
            workflow_state_json(500000009, "Done", "done", 2),
        ]
    )]);

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&workflows_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut list_args = make_list_args();
    list_args.owner = Some("@alice".to_string());
    let args = story::StoryArgs {
        action: story::StoryAction::List(Box::new(list_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_stories_with_state_filter() {
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
    )]);

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&workflows_body))
        .expect(2)
        .mount(&server)
        .await;

    let body = serde_json::json!([story_json(1, "In Progress Story", None)]);

    Mock::given(method("POST"))
        .and(path("/api/v3/stories/search"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut list_args = make_list_args();
    list_args.state = Some("in_progress".to_string());
    let args = story::StoryArgs {
        action: story::StoryAction::List(Box::new(list_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_stories_with_descriptions() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = serde_json::json!([story_json(1, "Story One", Some("First description")),]);

    Mock::given(method("POST"))
        .and(path("/api/v3/stories/search"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let workflows_body = serde_json::json!([workflow_json(
        500000006,
        "Default",
        vec![
            workflow_state_json(500000007, "Unstarted", "unstarted", 0),
            workflow_state_json(500000008, "In Progress", "started", 1),
            workflow_state_json(500000009, "Done", "done", 2),
        ]
    )]);

    Mock::given(method("GET"))
        .and(path("/api/v3/workflows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&workflows_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut list_args = make_list_args();
    list_args.desc = true;
    let args = story::StoryArgs {
        action: story::StoryAction::List(Box::new(list_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_stories_api_error() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("POST"))
        .and(path("/api/v3/stories/search"))
        .respond_with(ResponseTemplate::new(400))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let list_args = make_list_args();
    let args = story::StoryArgs {
        action: story::StoryAction::List(Box::new(list_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());
}
