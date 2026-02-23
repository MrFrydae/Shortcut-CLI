use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::make_update_args;
use crate::support::{epic_state_json, epic_workflow_json, full_epic_json};
use shortcut_cli::output::{ColorMode, OutputConfig, OutputMode};
use shortcut_cli::{api, commands::epic};

#[tokio::test]
async fn update_epic_sets_description() {
    let out = crate::support::make_output();
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
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_epic_api_error() {
    let out = crate::support::make_output();
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
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());
}

// --- Epic state resolution tests ---

#[tokio::test]
async fn update_epic_numeric_state_id_passes_through() {
    let out = crate::support::make_output();
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
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_epic_resolves_state_name_via_api() {
    let out = crate::support::make_output();
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
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_epic_state_cache_hit_skips_api() {
    let out = crate::support::make_output();
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
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_epic_unknown_state_name_errors() {
    let out = crate::support::make_output();
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
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Unknown epic state"));
    assert!(err.contains("To Do"));
    assert!(err.contains("In Progress"));
}

// --- --unless-state tests ---

fn standard_epic_workflow_body() -> serde_json::Value {
    epic_workflow_json(vec![
        epic_state_json(500000010, "To Do", "unstarted", 0),
        epic_state_json(500000011, "In Progress", "started", 1),
        epic_state_json(500000012, "Done", "done", 2),
    ])
}

fn full_epic_json_with_state(id: i64, name: &str, desc: &str, state_id: i64) -> serde_json::Value {
    let mut epic = full_epic_json(id, name, desc);
    epic["epic_state_id"] = serde_json::json!(state_id);
    epic
}

#[tokio::test]
async fn unless_state_skips_when_matched() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Epic is currently in "Done" (state_id 500000012)
    let get_body = full_epic_json_with_state(42, "My Epic", "desc", 500000012);
    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/epic-workflow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&standard_epic_workflow_body()))
        .mount(&server)
        .await;

    // PUT should NOT be called
    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.epic_state_id = Some("In Progress".to_string());
    update_args.unless_state = vec!["Done".to_string()];
    let args = epic::EpicArgs {
        action: epic::EpicAction::Update(Box::new(update_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn unless_state_proceeds_when_not_matched() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Epic is currently in "To Do" (state_id 500000010)
    let get_body = full_epic_json_with_state(42, "My Epic", "desc", 500000010);
    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/epic-workflow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&standard_epic_workflow_body()))
        .mount(&server)
        .await;

    let put_body = full_epic_json(42, "My Epic", "desc");
    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&put_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.epic_state_id = Some("In Progress".to_string());
    update_args.unless_state = vec!["Done".to_string()];
    let args = epic::EpicArgs {
        action: epic::EpicAction::Update(Box::new(update_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn unless_state_multiple_values() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Epic is currently in "In Progress" (state_id 500000011)
    let get_body = full_epic_json_with_state(42, "My Epic", "desc", 500000011);
    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/epic-workflow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&standard_epic_workflow_body()))
        .mount(&server)
        .await;

    // PUT should NOT be called
    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.epic_state_id = Some("Done".to_string());
    update_args.unless_state = vec!["In Progress".to_string(), "Done".to_string()];
    let args = epic::EpicArgs {
        action: epic::EpicAction::Update(Box::new(update_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn unless_state_normalizes_names() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Epic is currently in "In Progress" (state_id 500000011)
    let get_body = full_epic_json_with_state(42, "My Epic", "desc", 500000011);
    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/epic-workflow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&standard_epic_workflow_body()))
        .mount(&server)
        .await;

    // PUT should NOT be called — "in_progress" normalizes to match "In Progress"
    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.epic_state_id = Some("Done".to_string());
    update_args.unless_state = vec!["in_progress".to_string()];
    let args = epic::EpicArgs {
        action: epic::EpicAction::Update(Box::new(update_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn unless_state_json_output() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Json, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Epic is currently in "Done" (state_id 500000012)
    let get_body = full_epic_json_with_state(42, "My Epic", "desc", 500000012);
    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/epic-workflow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&standard_epic_workflow_body()))
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.epic_state_id = Some("In Progress".to_string());
    update_args.unless_state = vec!["Done".to_string()];
    let args = epic::EpicArgs {
        action: epic::EpicAction::Update(Box::new(update_args)),
    };
    let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(json["id"], 42);
    assert_eq!(json["skipped"], true);
    assert_eq!(json["current_state"], "Done");
    assert!(json["reason"].as_str().unwrap().contains("--unless-state"));
}

#[tokio::test]
async fn update_epic_state_name_normalization_variants() {
    let out = crate::support::make_output();
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
            .named(format!("workflow for '{variant}'"))
            .mount(&server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/api/v3/epics/42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&update_body))
            .expect(1)
            .named(format!("update for '{variant}'"))
            .mount(&server)
            .await;

        let client = api::client_with_token("test-token", &server.uri()).unwrap();
        let mut update_args = make_update_args(42);
        update_args.epic_state_id = Some(variant.to_string());
        let args = epic::EpicArgs {
            action: epic::EpicAction::Update(Box::new(update_args)),
        };
        let result = epic::run(&args, &client, tmp.path().to_path_buf(), &out).await;
        assert!(result.is_ok(), "Failed for variant '{variant}': {result:?}");

        server.reset().await;
    }
}
