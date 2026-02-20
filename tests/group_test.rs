mod support;

use sc::{api, commands::group};
use support::{default_icon, group_json, member_json, story_json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const UUID_GROUP1: &str = "11111111-1111-1111-1111-111111111111";
const UUID_GROUP2: &str = "22222222-2222-2222-2222-222222222222";
const UUID_ALICE: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

// --- List tests ---

#[tokio::test]
async fn list_groups() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        group_json(UUID_GROUP1, "Backend", "backend"),
        group_json(UUID_GROUP2, "Frontend", "frontend"),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::List { archived: false },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_groups_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::List { archived: false },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_groups_filters_archived() {
    let server = MockServer::start().await;

    let mut archived_group = group_json(UUID_GROUP2, "Old Team", "old-team");
    archived_group["archived"] = serde_json::Value::Bool(true);

    let body = serde_json::json!([
        group_json(UUID_GROUP1, "Active Team", "active-team"),
        archived_group,
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::List { archived: false },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

// --- Create tests ---

#[tokio::test]
async fn create_group_minimal() {
    let server = MockServer::start().await;

    let body = group_json(UUID_GROUP1, "New Team", "new-team");

    Mock::given(method("POST"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Create(Box::new(group::CreateArgs {
            name: "New Team".to_string(),
            mention_name: "new-team".to_string(),
            description: None,
            color: None,
            member_ids: vec![],
            workflow_ids: vec![],
        })),
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_group_with_members() {
    let server = MockServer::start().await;

    // Mock members endpoint for @mention resolution
    let members_body = serde_json::json!([member_json(
        UUID_ALICE,
        "alice",
        "Alice Smith",
        "member",
        false,
        Some(default_icon())
    ),]);
    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&members_body))
        .expect(1)
        .mount(&server)
        .await;

    let mut group = group_json(UUID_GROUP1, "Dev Team", "dev-team");
    group["member_ids"] = serde_json::json!([UUID_ALICE]);

    Mock::given(method("POST"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&group))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Create(Box::new(group::CreateArgs {
            name: "Dev Team".to_string(),
            mention_name: "dev-team".to_string(),
            description: None,
            color: None,
            member_ids: vec!["@alice".to_string()],
            workflow_ids: vec![],
        })),
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

// --- Get tests ---

#[tokio::test]
async fn get_group() {
    let server = MockServer::start().await;

    let mut group = group_json(UUID_GROUP1, "Backend", "backend");
    group["description"] = serde_json::json!("The backend team");
    group["num_stories"] = serde_json::json!(10);
    group["num_stories_started"] = serde_json::json!(3);
    group["num_stories_backlog"] = serde_json::json!(5);
    group["num_epics_started"] = serde_json::json!(2);
    group["member_ids"] = serde_json::json!([UUID_ALICE]);

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/groups/{UUID_GROUP1}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&group))
        .expect(1)
        .mount(&server)
        .await;

    // Mock members endpoint for member info display
    let members_body = serde_json::json!([member_json(
        UUID_ALICE,
        "alice",
        "Alice Smith",
        "member",
        false,
        Some(default_icon())
    ),]);
    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&members_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Get {
            id: UUID_GROUP1.to_string(),
        },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_group_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/groups/{UUID_GROUP1}")))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Get {
            id: UUID_GROUP1.to_string(),
        },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
}

// --- Update tests ---

#[tokio::test]
async fn update_group() {
    let server = MockServer::start().await;

    let body = group_json(UUID_GROUP1, "Renamed Team", "backend");

    Mock::given(method("PUT"))
        .and(path(format!("/api/v3/groups/{UUID_GROUP1}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Update(Box::new(group::UpdateArgs {
            id: UUID_GROUP1.to_string(),
            name: Some("Renamed Team".to_string()),
            mention_name: None,
            description: None,
            archived: None,
            color: None,
            member_ids: vec![],
            workflow_ids: vec![],
        })),
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_group_with_members() {
    let server = MockServer::start().await;

    // Mock members endpoint for @mention resolution
    let members_body = serde_json::json!([member_json(
        UUID_ALICE,
        "alice",
        "Alice Smith",
        "member",
        false,
        Some(default_icon())
    ),]);
    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&members_body))
        .expect(1)
        .mount(&server)
        .await;

    let mut group = group_json(UUID_GROUP1, "Backend", "backend");
    group["member_ids"] = serde_json::json!([UUID_ALICE]);

    Mock::given(method("PUT"))
        .and(path(format!("/api/v3/groups/{UUID_GROUP1}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&group))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Update(Box::new(group::UpdateArgs {
            id: UUID_GROUP1.to_string(),
            name: None,
            mention_name: None,
            description: None,
            archived: None,
            color: None,
            member_ids: vec!["@alice".to_string()],
            workflow_ids: vec![],
        })),
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

// --- Get by mention name tests ---

#[tokio::test]
async fn get_group_by_mention_name() {
    let server = MockServer::start().await;

    // Mock list-groups for cache population (mention name resolution)
    let list_body = serde_json::json!([
        group_json(UUID_GROUP1, "Backend", "backend"),
        group_json(UUID_GROUP2, "Frontend", "frontend"),
    ]);
    Mock::given(method("GET"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&list_body))
        .expect(1)
        .mount(&server)
        .await;

    // Mock get-group for the resolved UUID
    let mut detail = group_json(UUID_GROUP1, "Backend", "backend");
    detail["description"] = serde_json::json!("The backend team");
    detail["num_stories"] = serde_json::json!(5);
    detail["num_stories_started"] = serde_json::json!(2);
    detail["num_stories_backlog"] = serde_json::json!(3);
    detail["num_epics_started"] = serde_json::json!(1);
    detail["member_ids"] = serde_json::json!([UUID_ALICE]);

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/groups/{UUID_GROUP1}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&detail))
        .expect(1)
        .mount(&server)
        .await;

    // Mock members endpoint for member info display
    let members_body = serde_json::json!([member_json(
        UUID_ALICE,
        "alice",
        "Alice Smith",
        "member",
        false,
        Some(default_icon())
    ),]);
    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&members_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Get {
            id: "@backend".to_string(),
        },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_group_by_mention_name_not_found() {
    let server = MockServer::start().await;

    let list_body = serde_json::json!([group_json(UUID_GROUP1, "Backend", "backend"),]);
    Mock::given(method("GET"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&list_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Get {
            id: "@nonexistent".to_string(),
        },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
}

// --- Stories tests ---

#[tokio::test]
async fn list_group_stories() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        story_json(1, "Story One", None),
        story_json(2, "Story Two", None),
    ]);

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/groups/{UUID_GROUP1}/stories")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Stories {
            id: UUID_GROUP1.to_string(),
            limit: None,
            offset: None,
            desc: false,
        },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_group_stories_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/groups/{UUID_GROUP1}/stories")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Stories {
            id: UUID_GROUP1.to_string(),
            limit: None,
            offset: None,
            desc: false,
        },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn stories_by_mention_name() {
    let server = MockServer::start().await;

    // Mock list-groups for mention name resolution
    let list_body = serde_json::json!([group_json(UUID_GROUP1, "Backend", "backend"),]);
    Mock::given(method("GET"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&list_body))
        .expect(1)
        .mount(&server)
        .await;

    // Mock stories endpoint for the resolved UUID
    let stories_body = serde_json::json!([
        story_json(1, "Story One", None),
        story_json(2, "Story Two", None),
    ]);
    Mock::given(method("GET"))
        .and(path(format!("/api/v3/groups/{UUID_GROUP1}/stories")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&stories_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Stories {
            id: "@backend".to_string(),
            limit: None,
            offset: None,
            desc: false,
        },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}
