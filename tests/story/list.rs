use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{default_icon, member_json, story_json, workflow_json, workflow_state_json};
use crate::{UUID_ALICE, make_list_args};
use sc::{api, commands::story};

#[tokio::test]
async fn list_stories_minimal() {
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

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let list_args = make_list_args();
    let args = story::StoryArgs {
        action: story::StoryAction::List(Box::new(list_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_stories_empty() {
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
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_stories_with_owner_filter() {
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

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut list_args = make_list_args();
    list_args.owner = Some("@alice".to_string());
    let args = story::StoryArgs {
        action: story::StoryAction::List(Box::new(list_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_stories_with_state_filter() {
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
        .expect(1)
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
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_stories_with_descriptions() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = serde_json::json!([story_json(1, "Story One", Some("First description")),]);

    Mock::given(method("POST"))
        .and(path("/api/v3/stories/search"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut list_args = make_list_args();
    list_args.desc = true;
    let args = story::StoryArgs {
        action: story::StoryAction::List(Box::new(list_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_stories_api_error() {
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
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
}
