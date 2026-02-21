mod support;

use sc::{api, commands::story, commands::story_link};
use support::{
    custom_field_enum_value_json, custom_field_json, default_icon, full_story_json,
    full_story_json_with_custom_fields, full_story_json_with_links, member_json,
    story_custom_field_json, story_json, story_link_json, typed_story_link_json, workflow_json,
    workflow_state_json,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const UUID_ALICE: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const UUID_BOB: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";

const UUID_FIELD_1: &str = "11111111-1111-1111-1111-111111111111";
const UUID_VAL_A: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaab";
const UUID_VAL_B: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb2";

fn make_create_args(name: &str) -> story::CreateArgs {
    story::CreateArgs {
        name: name.to_string(),
        description: None,
        story_type: None,
        owner: vec![],
        state: None,
        epic_id: None,
        estimate: None,
        labels: vec![],
        iteration_id: None,
        custom_fields: vec![],
    }
}

fn make_update_args(id: i64) -> story::UpdateArgs {
    story::UpdateArgs {
        id,
        name: None,
        description: None,
        story_type: None,
        owner: vec![],
        state: None,
        epic_id: None,
        estimate: None,
        labels: vec![],
        iteration_id: None,
        custom_fields: vec![],
    }
}

fn make_list_args() -> story::ListArgs {
    story::ListArgs {
        owner: None,
        state: None,
        epic_id: None,
        story_type: None,
        label: None,
        project_id: None,
        limit: 25,
        desc: false,
    }
}

// --- Create tests ---

#[tokio::test]
async fn create_story_minimal() {
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
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_story_with_owner_mention() {
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
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_story_with_state_name() {
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
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

// --- Update tests ---

#[tokio::test]
async fn update_story_name_and_description() {
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
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_story_state() {
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
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_story_owner() {
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
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

// --- Get tests ---

#[tokio::test]
async fn get_story_prints_details() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_story_json(99, "Important Story", "Some description");

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Get { id: 99 },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

// --- State resolution tests ---

#[tokio::test]
async fn state_resolution_numeric_passes_through() {
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
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn state_resolution_cache_hit() {
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
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn state_ambiguous_error() {
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
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Ambiguous"));
}

#[tokio::test]
async fn unknown_state_name_errors() {
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
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Unknown workflow state"));
    assert!(err.contains("Unstarted"));
    assert!(err.contains("In Progress"));
}

// --- List tests ---

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

// --- Delete tests ---

#[tokio::test]
async fn delete_story_with_confirm() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_story_json(42, "My Story", "desc");

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Delete {
            id: 42,
            confirm: true,
        },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_story_without_confirm_errors() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Delete {
            id: 42,
            confirm: false,
        },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}

#[tokio::test]
async fn delete_story_not_found() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/999"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Delete {
            id: 999,
            confirm: true,
        },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
}

// --- Story Link tests ---

#[tokio::test]
async fn link_create_blocks() {
    let server = MockServer::start().await;

    let body = story_link_json(1, 10, 20, "blocks");

    Mock::given(method("POST"))
        .and(path("/api/v3/story-links"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_link::LinkArgs {
        action: story_link::LinkAction::Create(story_link::CreateLinkArgs {
            subject: 10,
            object: 20,
            verb: "blocks".to_string(),
        }),
    };
    let result = story_link::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn link_create_blocked_by_swaps() {
    let server = MockServer::start().await;

    // When verb is "blocked-by", subject/object are swapped:
    // user says "10 blocked-by 20" -> API gets "20 blocks 10"
    let body = story_link_json(2, 20, 10, "blocks");

    Mock::given(method("POST"))
        .and(path("/api/v3/story-links"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_link::LinkArgs {
        action: story_link::LinkAction::Create(story_link::CreateLinkArgs {
            subject: 10,
            object: 20,
            verb: "blocked-by".to_string(),
        }),
    };
    let result = story_link::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn link_create_relates_to() {
    let server = MockServer::start().await;

    let body = story_link_json(3, 10, 20, "relates to");

    Mock::given(method("POST"))
        .and(path("/api/v3/story-links"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_link::LinkArgs {
        action: story_link::LinkAction::Create(story_link::CreateLinkArgs {
            subject: 10,
            object: 20,
            verb: "relates-to".to_string(),
        }),
    };
    let result = story_link::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn link_list_shows_links() {
    let server = MockServer::start().await;

    let links = vec![
        typed_story_link_json(1, 42, 99, "blocks", "subject"),
        typed_story_link_json(2, 50, 42, "duplicates", "object"),
    ];
    let story_body = full_story_json_with_links(42, "My Story", "desc", links);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&story_body))
        .expect(1)
        .mount(&server)
        .await;

    // Mock fetches for linked story names
    let story_99 = full_story_json(99, "Blocked Story", "");
    Mock::given(method("GET"))
        .and(path("/api/v3/stories/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&story_99))
        .expect(1)
        .mount(&server)
        .await;

    let story_50 = full_story_json(50, "Duplicate Source", "");
    Mock::given(method("GET"))
        .and(path("/api/v3/stories/50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&story_50))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_link::LinkArgs {
        action: story_link::LinkAction::List { story_id: 42 },
    };
    let result = story_link::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn link_list_empty() {
    let server = MockServer::start().await;

    let story_body = full_story_json(42, "My Story", "desc");

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&story_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_link::LinkArgs {
        action: story_link::LinkAction::List { story_id: 42 },
    };
    let result = story_link::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn link_delete_with_confirm() {
    let server = MockServer::start().await;

    let link_body = story_link_json(5, 10, 20, "blocks");

    Mock::given(method("GET"))
        .and(path("/api/v3/story-links/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&link_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/story-links/5"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_link::LinkArgs {
        action: story_link::LinkAction::Delete {
            id: 5,
            confirm: true,
        },
    };
    let result = story_link::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn link_delete_without_confirm_errors() {
    let server = MockServer::start().await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_link::LinkArgs {
        action: story_link::LinkAction::Delete {
            id: 5,
            confirm: false,
        },
    };
    let result = story_link::run(&args, &client).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}

#[tokio::test]
async fn get_story_with_links() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let links = vec![
        typed_story_link_json(1, 42, 99, "blocks", "subject"),
        typed_story_link_json(2, 50, 42, "relates to", "object"),
    ];
    let body = full_story_json_with_links(42, "Linked Story", "A story with links", links);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Get { id: 42 },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

// --- Custom field tests ---

#[tokio::test]
async fn get_story_with_custom_fields() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let custom_fields = vec![story_custom_field_json(UUID_FIELD_1, UUID_VAL_A, "High")];
    let body = full_story_json_with_custom_fields(
        42,
        "CF Story",
        "A story with custom fields",
        custom_fields,
    );

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    // Mock custom fields list for name resolution
    let cf_body = serde_json::json!([custom_field_json(
        UUID_FIELD_1,
        "Priority",
        vec![
            custom_field_enum_value_json(UUID_VAL_A, "High", 0, true),
            custom_field_enum_value_json(UUID_VAL_B, "Low", 1, true),
        ]
    )]);

    Mock::given(method("GET"))
        .and(path("/api/v3/custom-fields"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&cf_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Get { id: 42 },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_story_with_custom_field() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Mock custom fields list for resolution
    let cf_body = serde_json::json!([custom_field_json(
        UUID_FIELD_1,
        "Priority",
        vec![
            custom_field_enum_value_json(UUID_VAL_A, "High", 0, true),
            custom_field_enum_value_json(UUID_VAL_B, "Low", 1, true),
        ]
    )]);

    Mock::given(method("GET"))
        .and(path("/api/v3/custom-fields"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&cf_body))
        .expect(1)
        .mount(&server)
        .await;

    let body = full_story_json(200, "CF Create Story", "");

    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("CF Create Story");
    create_args.custom_fields = vec!["Priority=High".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_story_with_custom_field() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Mock custom fields list for resolution
    let cf_body = serde_json::json!([custom_field_json(
        UUID_FIELD_1,
        "Priority",
        vec![
            custom_field_enum_value_json(UUID_VAL_A, "High", 0, true),
            custom_field_enum_value_json(UUID_VAL_B, "Low", 1, true),
        ]
    )]);

    Mock::given(method("GET"))
        .and(path("/api/v3/custom-fields"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&cf_body))
        .expect(1)
        .mount(&server)
        .await;

    let body = full_story_json(42, "Updated Story", "desc");

    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.custom_fields = vec!["Priority=Low".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_story_custom_field_invalid_format() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("Bad CF Story");
    create_args.custom_fields = vec!["NoEquals".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("FieldName=Value"));
}

#[tokio::test]
async fn create_story_custom_field_unknown_field() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Mock custom fields list — field we ask for doesn't exist
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

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("Bad CF Story");
    create_args.custom_fields = vec!["Nonexistent=High".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Unknown custom field"));
    assert!(err.contains("Priority"));
}

#[tokio::test]
async fn create_story_custom_field_unknown_value() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Mock custom fields list — value we ask for doesn't exist
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

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("Bad CF Story");
    create_args.custom_fields = vec!["Priority=Bogus".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Unknown value"));
    assert!(err.contains("High"));
}
