use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{
    default_icon, history_action_branch_create_json, history_action_comment_create_json,
    history_action_label_create_json, history_action_story_create_json,
    history_action_story_update_json, history_action_task_create_json, history_entry_json,
    history_reference_label_json, history_reference_workflow_state_json, member_json,
};
use crate::{UUID_ALICE, UUID_BOB};
use sc::{api, commands::story};

fn make_history_args(id: i64, limit: Option<usize>) -> story::HistoryArgs {
    story::HistoryArgs { id, limit }
}

#[tokio::test]
async fn history_basic_story_create() {
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

    let history_body = serde_json::json!([history_entry_json(
        "11111111-1111-1111-1111-111111111111",
        "2024-01-01T00:00:00Z",
        Some(UUID_ALICE),
        vec![history_action_story_create_json(42, "Fix login bug", "bug")],
        vec![],
    ),]);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42/history"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&history_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let history_args = make_history_args(42, None);
    let args = story::StoryArgs {
        action: story::StoryAction::History(history_args),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn history_story_update_state_change() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let members_body = serde_json::json!([member_json(
        UUID_BOB,
        "bob",
        "Bob Jones",
        "member",
        false,
        None
    ),]);

    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&members_body))
        .expect(1)
        .mount(&server)
        .await;

    let changes = serde_json::json!({
        "workflow_state_id": { "old": 500000007, "new": 500000008 }
    });
    let history_body = serde_json::json!([history_entry_json(
        "22222222-2222-2222-2222-222222222222",
        "2024-01-01T01:00:00Z",
        Some(UUID_BOB),
        vec![history_action_story_update_json(
            42,
            "Fix login bug",
            "bug",
            changes
        )],
        vec![
            history_reference_workflow_state_json(500000007, "Backlog", "unstarted"),
            history_reference_workflow_state_json(500000008, "In Progress", "started"),
        ],
    ),]);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42/history"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&history_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let history_args = make_history_args(42, None);
    let args = story::StoryArgs {
        action: story::StoryAction::History(history_args),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn history_story_update_multiple_changes() {
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

    let changes = serde_json::json!({
        "name": { "old": "Old Title", "new": "New Title" },
        "estimate": { "old": null, "new": 5 },
        "label_ids": { "adds": [10], "removes": [] },
        "owner_ids": { "adds": [UUID_ALICE], "removes": [] }
    });
    let history_body = serde_json::json!([history_entry_json(
        "33333333-3333-3333-3333-333333333333",
        "2024-01-01T03:00:00Z",
        Some(UUID_ALICE),
        vec![history_action_story_update_json(
            42,
            "New Title",
            "feature",
            changes
        )],
        vec![history_reference_label_json(10, "urgent"),],
    ),]);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42/history"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&history_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let history_args = make_history_args(42, None);
    let args = story::StoryArgs {
        action: story::StoryAction::History(history_args),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn history_with_limit() {
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

    let history_body = serde_json::json!([
        history_entry_json(
            "11111111-1111-1111-1111-111111111111",
            "2024-01-01T00:00:00Z",
            Some(UUID_ALICE),
            vec![history_action_story_create_json(42, "Story", "feature")],
            vec![],
        ),
        history_entry_json(
            "22222222-2222-2222-2222-222222222222",
            "2024-01-01T01:00:00Z",
            Some(UUID_ALICE),
            vec![history_action_task_create_json(1, "Task one")],
            vec![],
        ),
        history_entry_json(
            "33333333-3333-3333-3333-333333333333",
            "2024-01-01T02:00:00Z",
            Some(UUID_ALICE),
            vec![history_action_task_create_json(2, "Task two")],
            vec![],
        ),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42/history"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&history_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let history_args = make_history_args(42, Some(1));
    let args = story::StoryArgs {
        action: story::StoryAction::History(history_args),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn history_empty() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42/history"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let history_args = make_history_args(42, None);
    let args = story::StoryArgs {
        action: story::StoryAction::History(history_args),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn history_api_error() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/999/history"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let history_args = make_history_args(999, None);
    let args = story::StoryArgs {
        action: story::StoryAction::History(history_args),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn history_multiple_action_types() {
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

    let changes = serde_json::json!({
        "workflow_state_id": { "old": 500000007, "new": 500000008 }
    });

    let history_body = serde_json::json!([
        history_entry_json(
            "11111111-1111-1111-1111-111111111111",
            "2024-01-01T00:00:00Z",
            Some(UUID_ALICE),
            vec![history_action_story_create_json(42, "My Story", "feature")],
            vec![],
        ),
        history_entry_json(
            "22222222-2222-2222-2222-222222222222",
            "2024-01-01T01:00:00Z",
            Some(UUID_ALICE),
            vec![history_action_story_update_json(
                42, "My Story", "feature", changes
            )],
            vec![
                history_reference_workflow_state_json(500000007, "Backlog", "unstarted"),
                history_reference_workflow_state_json(500000008, "In Progress", "started"),
            ],
        ),
        history_entry_json(
            "33333333-3333-3333-3333-333333333333",
            "2024-01-01T02:00:00Z",
            Some(UUID_ALICE),
            vec![history_action_task_create_json(1, "Write tests")],
            vec![],
        ),
        history_entry_json(
            "44444444-4444-4444-4444-444444444444",
            "2024-01-01T03:00:00Z",
            Some(UUID_ALICE),
            vec![history_action_comment_create_json(10, UUID_ALICE)],
            vec![],
        ),
        history_entry_json(
            "55555555-5555-5555-5555-555555555555",
            "2024-01-01T04:00:00Z",
            Some(UUID_ALICE),
            vec![history_action_label_create_json(20, "urgent")],
            vec![],
        ),
        history_entry_json(
            "66666666-6666-6666-6666-666666666666",
            "2024-01-01T05:00:00Z",
            Some(UUID_ALICE),
            vec![history_action_branch_create_json(30, "fix/login-bug")],
            vec![],
        ),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42/history"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&history_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let history_args = make_history_args(42, None);
    let args = story::StoryArgs {
        action: story::StoryAction::History(history_args),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn history_member_cache_hit() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Pre-populate member cache so no /members API call is needed
    let cache: std::collections::HashMap<String, String> =
        [("alice".to_string(), UUID_ALICE.to_string())].into();
    let cache_path = tmp.path().join("member_cache.json");
    std::fs::write(&cache_path, serde_json::to_string(&cache).unwrap()).unwrap();

    // No /members mock — cache hit should avoid API call

    let history_body = serde_json::json!([history_entry_json(
        "11111111-1111-1111-1111-111111111111",
        "2024-01-01T00:00:00Z",
        Some(UUID_ALICE),
        vec![history_action_story_create_json(
            42,
            "Cached Story",
            "feature"
        )],
        vec![],
    ),]);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42/history"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&history_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let history_args = make_history_args(42, None);
    let args = story::StoryArgs {
        action: story::StoryAction::History(history_args),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}
