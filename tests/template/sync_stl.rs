use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{full_epic_json, full_story_json, make_output, mount_default_workflow};
use shortcut_cli::output::{ColorMode, OutputConfig, OutputMode};
use shortcut_cli::stl::state;
use shortcut_cli::{api, commands::template};

fn sync_args(file: &str) -> template::TemplateArgs {
    template::TemplateArgs {
        action: template::TemplateAction::Sync(Box::new(template::sync_stl::SyncArgs {
            file: file.to_string(),
            state: None,
            confirm: true,
            prune: false,
            vars: vec![],
        })),
    }
}

fn sync_args_with_state(file: &str, state_path: &str) -> template::TemplateArgs {
    template::TemplateArgs {
        action: template::TemplateAction::Sync(Box::new(template::sync_stl::SyncArgs {
            file: file.to_string(),
            state: Some(state_path.to_string()),
            confirm: true,
            prune: false,
            vars: vec![],
        })),
    }
}

fn sync_args_with_prune(file: &str, state_path: &str) -> template::TemplateArgs {
    template::TemplateArgs {
        action: template::TemplateAction::Sync(Box::new(template::sync_stl::SyncArgs {
            file: file.to_string(),
            state: Some(state_path.to_string()),
            confirm: true,
            prune: true,
            vars: vec![],
        })),
    }
}

fn write_template(dir: &tempfile::TempDir, yaml: &str) -> String {
    let p = dir.path().join("test.shortcut.yml");
    std::fs::write(&p, yaml).unwrap();
    p.to_str().unwrap().to_string()
}

// ── First sync creates resources and writes state ──

#[tokio::test]
async fn first_sync_creates_and_writes_state() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    mount_default_workflow(&server).await;

    let epic_resp = full_epic_json(55, "My Epic", "");
    Mock::given(method("POST"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&epic_resp))
        .expect(1)
        .mount(&server)
        .await;

    let story_resp = full_story_json(200, "My Story", "");
    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&story_resp))
        .expect(1)
        .mount(&server)
        .await;

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: epic
    alias: my-epic
    fields:
      name: "My Epic"
  - action: create
    entity: story
    alias: my-story
    fields:
      name: "My Story"
      epic_id: $ref(my-epic)
"#;
    let file = write_template(&tmp, yaml);
    let state_path = format!("{}.state.json", file);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = sync_args_with_state(&file, &state_path);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");

    // Verify state file was written
    let loaded = state::load_state(std::path::Path::new(&state_path))
        .unwrap()
        .expect("state file should exist");
    assert_eq!(loaded.resources.len(), 2);
    assert!(loaded.resources.contains_key("my-epic"));
    assert!(loaded.resources.contains_key("my-story"));
}

// ── Second sync sends PATCH not POST ──

#[tokio::test]
async fn second_sync_updates_existing() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // No POST expected — only PUT (update)
    let epic_resp = full_epic_json(55, "Updated Epic", "");
    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/55"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&epic_resp))
        .expect(1)
        .mount(&server)
        .await;

    // Write existing state
    let state_path = tmp.path().join("test.state.json");
    let mut existing = state::SyncState::new();
    existing.resources.insert(
        "my-epic".to_string(),
        state::ResourceState::Single {
            entity: "epic".to_string(),
            id: serde_json::json!(55),
            tasks: None,
        },
    );
    state::save_state(&existing, &state_path).unwrap();

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: epic
    alias: my-epic
    fields:
      name: "Updated Epic"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = sync_args_with_state(&file, state_path.to_str().unwrap());
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");
}

// ── New operation added to template creates it ──

#[tokio::test]
async fn new_operation_in_template_creates_it() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    mount_default_workflow(&server).await;

    let epic_resp = full_epic_json(55, "Existing Epic", "");
    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/55"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&epic_resp))
        .expect(1)
        .mount(&server)
        .await;

    let story_resp = full_story_json(300, "New Story", "");
    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&story_resp))
        .expect(1)
        .mount(&server)
        .await;

    let state_path = tmp.path().join("test.state.json");
    let mut existing = state::SyncState::new();
    existing.resources.insert(
        "my-epic".to_string(),
        state::ResourceState::Single {
            entity: "epic".to_string(),
            id: serde_json::json!(55),
            tasks: None,
        },
    );
    state::save_state(&existing, &state_path).unwrap();

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: epic
    alias: my-epic
    fields:
      name: "Existing Epic"
  - action: create
    entity: story
    alias: new-story
    fields:
      name: "New Story"
      epic_id: $ref(my-epic)
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = sync_args_with_state(&file, state_path.to_str().unwrap());
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");

    let loaded = state::load_state(&state_path).unwrap().unwrap();
    assert_eq!(loaded.resources.len(), 2);
    assert!(loaded.resources.contains_key("new-story"));
}

// ── Orphan warning printed (no API delete call) ──

#[tokio::test]
async fn orphan_warning_without_prune() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let epic_resp = full_epic_json(55, "Kept Epic", "");
    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/55"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&epic_resp))
        .expect(1)
        .mount(&server)
        .await;

    // No DELETE expected since prune is off
    Mock::given(method("DELETE"))
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .mount(&server)
        .await;

    let state_path = tmp.path().join("test.state.json");
    let mut existing = state::SyncState::new();
    existing.resources.insert(
        "kept-epic".to_string(),
        state::ResourceState::Single {
            entity: "epic".to_string(),
            id: serde_json::json!(55),
            tasks: None,
        },
    );
    existing.resources.insert(
        "removed-story".to_string(),
        state::ResourceState::Single {
            entity: "story".to_string(),
            id: serde_json::json!(99),
            tasks: None,
        },
    );
    state::save_state(&existing, &state_path).unwrap();

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: epic
    alias: kept-epic
    fields:
      name: "Kept Epic"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = sync_args_with_state(&file, state_path.to_str().unwrap());
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(
        output.contains("orphan"),
        "Expected orphan warning in output: {output}"
    );
    assert!(
        output.contains("--prune"),
        "Expected --prune hint in output: {output}"
    );
}

// ── --prune deletes orphans ──

#[tokio::test]
async fn prune_deletes_orphans() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let epic_resp = full_epic_json(55, "Kept Epic", "");
    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/55"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&epic_resp))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/stories/99"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let state_path = tmp.path().join("test.state.json");
    let mut existing = state::SyncState::new();
    existing.resources.insert(
        "kept-epic".to_string(),
        state::ResourceState::Single {
            entity: "epic".to_string(),
            id: serde_json::json!(55),
            tasks: None,
        },
    );
    existing.resources.insert(
        "removed-story".to_string(),
        state::ResourceState::Single {
            entity: "story".to_string(),
            id: serde_json::json!(99),
            tasks: None,
        },
    );
    state::save_state(&existing, &state_path).unwrap();

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: epic
    alias: kept-epic
    fields:
      name: "Kept Epic"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = sync_args_with_prune(&file, state_path.to_str().unwrap());
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");

    // Verify orphan removed from state
    let loaded = state::load_state(&state_path).unwrap().unwrap();
    assert!(!loaded.resources.contains_key("removed-story"));
}

// ── Repeat: existing key patched, new key created ──

#[tokio::test]
async fn repeat_existing_key_patched_new_key_created() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    mount_default_workflow(&server).await;

    // Update existing story
    let update_resp = full_story_json(100, "Updated Story", "Shared");
    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&update_resp))
        .expect(1)
        .mount(&server)
        .await;

    // Create new story
    let create_resp = full_story_json(200, "Brand New", "Shared");
    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&create_resp))
        .expect(1)
        .mount(&server)
        .await;

    let state_path = tmp.path().join("test.state.json");
    let mut existing = state::SyncState::new();
    let mut entries = std::collections::HashMap::new();
    entries.insert(
        "existing".to_string(),
        state::EntryState {
            id: serde_json::json!(100),
            tasks: None,
        },
    );
    existing.resources.insert(
        "stories".to_string(),
        state::ResourceState::Repeat {
            entity: "story".to_string(),
            entries,
        },
    );
    state::save_state(&existing, &state_path).unwrap();

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    alias: stories
    repeat:
      - key: existing
        name: "Updated Story"
      - key: brand-new
        name: "Brand New"
    fields:
      description: "Shared"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = sync_args_with_state(&file, state_path.to_str().unwrap());
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");

    let loaded = state::load_state(&state_path).unwrap().unwrap();
    match &loaded.resources["stories"] {
        state::ResourceState::Repeat { entries, .. } => {
            assert_eq!(entries.len(), 2);
            assert!(entries.contains_key("existing"));
            assert!(entries.contains_key("brand-new"));
        }
        _ => panic!("Expected Repeat resource"),
    }
}

// ── Skip already-applied side effect ──

#[tokio::test]
async fn skip_already_applied_comment() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // No API call expected
    Mock::given(method("POST"))
        .and(path("/api/v3/stories/500/comments"))
        .respond_with(ResponseTemplate::new(201))
        .expect(0)
        .mount(&server)
        .await;

    let state_path = tmp.path().join("test.state.json");
    let mut existing = state::SyncState::new();
    existing.applied.push("op-0-comment".to_string());
    state::save_state(&existing, &state_path).unwrap();

    let yaml = r#"
version: 1
operations:
  - action: comment
    entity: story
    id: 500
    fields:
      text: "Already posted"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = sync_args_with_state(&file, state_path.to_str().unwrap());
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(
        output.contains("Skipped"),
        "Expected skip message in output: {output}"
    );
}

// ── $ref() resolves correctly between synced operations ──

#[tokio::test]
async fn ref_resolves_across_synced_ops() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    mount_default_workflow(&server).await;

    let epic_resp = full_epic_json(55, "My Epic", "");
    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/55"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&epic_resp))
        .expect(1)
        .mount(&server)
        .await;

    // Story update must include epic_id: 55
    let story_resp = full_story_json(200, "My Story", "");
    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/200"))
        .and(body_partial_json(serde_json::json!({"epic_id": 55})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&story_resp))
        .expect(1)
        .mount(&server)
        .await;

    let state_path = tmp.path().join("test.state.json");
    let mut existing = state::SyncState::new();
    existing.resources.insert(
        "my-epic".to_string(),
        state::ResourceState::Single {
            entity: "epic".to_string(),
            id: serde_json::json!(55),
            tasks: None,
        },
    );
    existing.resources.insert(
        "my-story".to_string(),
        state::ResourceState::Single {
            entity: "story".to_string(),
            id: serde_json::json!(200),
            tasks: None,
        },
    );
    state::save_state(&existing, &state_path).unwrap();

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: epic
    alias: my-epic
    fields:
      name: "My Epic"
  - action: create
    entity: story
    alias: my-story
    fields:
      name: "My Story"
      epic_id: $ref(my-epic)
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = sync_args_with_state(&file, state_path.to_str().unwrap());
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");
}

// ── Validation: create without alias rejected ──

#[tokio::test]
async fn sync_rejects_create_without_alias() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    fields:
      name: "No alias"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = sync_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(
        output.contains("alias"),
        "Expected alias validation error: {output}"
    );
}

// ── Validation: repeat without key rejected ──

#[tokio::test]
async fn sync_rejects_repeat_without_key() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    alias: stories
    repeat:
      - name: "No key"
    fields:
      description: "Shared"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = sync_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(
        output.contains("key"),
        "Expected key validation error: {output}"
    );
}
