mod support;

use sc::{api, commands::epic};
use support::{
    default_icon, epic_json, epic_state_json, epic_workflow_json, full_epic_json, member_json,
};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const UUID_ALICE: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

fn make_list_args(desc: bool) -> epic::EpicArgs {
    epic::EpicArgs {
        action: epic::EpicAction::List { desc },
    }
}

fn make_update_args(id: i64) -> epic::UpdateArgs {
    epic::UpdateArgs {
        id,
        name: None,
        description: None,
        deadline: None,
        archived: None,
        epic_state_id: None,
        labels: vec![],
        objective_ids: vec![],
        owner_ids: vec![],
        follower_ids: vec![],
        requested_by_id: None,
    }
}

fn make_create_args(name: &str) -> epic::CreateArgs {
    epic::CreateArgs {
        name: name.to_string(),
        description: None,
        state: None,
        deadline: None,
        owners: vec![],
        labels: vec![],
        objective_ids: vec![],
        followers: vec![],
        requested_by: None,
    }
}

// --- List tests ---

#[tokio::test]
async fn list_epics_prints_names() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = serde_json::json!([
        epic_json(1, "Epic One", None),
        epic_json(2, "Epic Two", None),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_list_args(false);
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epics_with_descriptions() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = serde_json::json!([epic_json(1, "Epic One", Some("Description of epic one")),]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .and(query_param("includes_description", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_list_args(true);
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epics_empty() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = serde_json::json!([]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_list_args(false);
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epics_api_error() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(400))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_list_args(false);
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
}

// --- Update tests ---

#[tokio::test]
async fn update_epic_sets_description() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_epic_json(42, "My Epic", "new description");

    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.description = Some("new description".to_string());
    let args = epic::EpicArgs {
        action: epic::EpicAction::Update(Box::new(update_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_epic_api_error() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/99"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let update_args = make_update_args(99);
    let args = epic::EpicArgs {
        action: epic::EpicAction::Update(Box::new(update_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
}

// --- Epic state resolution tests ---

#[tokio::test]
async fn update_epic_numeric_state_id_passes_through() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_epic_json(42, "My Epic", "desc");

    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    // No epic-workflow mock — numeric ID should not trigger API call
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.epic_state_id = Some("500000042".to_string());
    let args = epic::EpicArgs {
        action: epic::EpicAction::Update(Box::new(update_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_epic_resolves_state_name_via_api() {
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

    let update_body = full_epic_json(42, "My Epic", "desc");

    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&update_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.epic_state_id = Some("in_progress".to_string());
    let args = epic::EpicArgs {
        action: epic::EpicAction::Update(Box::new(update_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_epic_state_cache_hit_skips_api() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Pre-populate cache
    let cache: std::collections::HashMap<String, i64> =
        [("in progress".to_string(), 500000011)].into();
    let cache_path = tmp.path().join("epic_state_cache.json");
    std::fs::write(&cache_path, serde_json::to_string(&cache).unwrap()).unwrap();

    let update_body = full_epic_json(42, "My Epic", "desc");

    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&update_body))
        .expect(1)
        .mount(&server)
        .await;

    // No epic-workflow mock — cache hit should avoid API call
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.epic_state_id = Some("in_progress".to_string());
    let args = epic::EpicArgs {
        action: epic::EpicAction::Update(Box::new(update_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_epic_unknown_state_name_errors() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let workflow_body = epic_workflow_json(vec![
        epic_state_json(500000010, "To Do", "unstarted", 0),
        epic_state_json(500000011, "In Progress", "started", 1),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epic-workflow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&workflow_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.epic_state_id = Some("nonexistent".to_string());
    let args = epic::EpicArgs {
        action: epic::EpicAction::Update(Box::new(update_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Unknown epic state"));
    assert!(err.contains("To Do"));
    assert!(err.contains("In Progress"));
}

#[tokio::test]
async fn update_epic_state_name_normalization_variants() {
    let server = MockServer::start().await;

    let workflow_body = epic_workflow_json(vec![
        epic_state_json(500000010, "To Do", "unstarted", 0),
        epic_state_json(500000011, "In Progress", "started", 1),
        epic_state_json(500000012, "Done", "done", 2),
    ]);

    let update_body = full_epic_json(42, "My Epic", "desc");

    for variant in [
        "IN_PROGRESS",
        "In-Progress",
        "in progress",
        "IN-PROGRESS",
        "in_progress",
    ] {
        let tmp = tempfile::tempdir().unwrap();

        Mock::given(method("GET"))
            .and(path("/api/v3/epic-workflow"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&workflow_body))
            .expect(1)
            .named(&format!("workflow for '{variant}'"))
            .mount(&server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/api/v3/epics/42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&update_body))
            .expect(1)
            .named(&format!("update for '{variant}'"))
            .mount(&server)
            .await;

        let client = api::client_with_token("test-token", &server.uri()).unwrap();
        let mut update_args = make_update_args(42);
        update_args.epic_state_id = Some(variant.to_string());
        let args = epic::EpicArgs {
            action: epic::EpicAction::Update(Box::new(update_args)),
        };
        let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
        assert!(result.is_ok(), "Failed for variant '{variant}': {result:?}");

        server.reset().await;
    }
}

// --- Create tests ---

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

// --- Get tests ---

#[tokio::test]
async fn get_epic_prints_details() {
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
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_epic_not_found() {
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
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn get_epic_with_cached_state() {
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
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

// --- Delete tests ---

#[tokio::test]
async fn delete_epic_with_confirm() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_epic_json(42, "My Epic", "desc");

    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic::EpicArgs {
        action: epic::EpicAction::Delete {
            id: 42,
            confirm: true,
        },
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_epic_without_confirm_errors() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic::EpicArgs {
        action: epic::EpicAction::Delete {
            id: 42,
            confirm: false,
        },
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}

#[tokio::test]
async fn delete_epic_not_found() {
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
        action: epic::EpicAction::Delete {
            id: 999,
            confirm: true,
        },
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
}
