use wiremock::matchers::{body_json_string, body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{full_story_json, make_dry_run_output, make_output};
use sc::output::{ColorMode, OutputConfig, OutputMode};
use sc::{api, commands::template};

fn run_args(file: &str) -> template::TemplateArgs {
    template::TemplateArgs {
        action: template::TemplateAction::Run(Box::new(template::run_stl::RunArgs {
            file: file.to_string(),
            confirm: true,
            vars: vec![],
        })),
    }
}

fn run_args_with_vars(file: &str, vars: Vec<(String, String)>) -> template::TemplateArgs {
    template::TemplateArgs {
        action: template::TemplateAction::Run(Box::new(template::run_stl::RunArgs {
            file: file.to_string(),
            confirm: true,
            vars,
        })),
    }
}

/// Helper to write a YAML template to a temp file and return its path.
fn write_template(dir: &tempfile::TempDir, yaml: &str) -> String {
    let path = dir.path().join("test.sc.yml");
    std::fs::write(&path, yaml).unwrap();
    path.to_str().unwrap().to_string()
}

/// Helper to build a simple API response for a created entity.
fn created_response(id: i64, name: &str, entity_type: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": name,
        "entity_type": entity_type
    })
}

#[tokio::test]
async fn run_create_story() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let response = full_story_json(200, "My Story", "");

    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&response))
        .expect(1)
        .mount(&server)
        .await;

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    fields:
      name: "My Story"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");
}

#[tokio::test]
async fn run_create_epic_then_story_with_ref() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let epic_response = created_response(55, "Auth Hardening", "epic");
    let story_response = full_story_json(200, "JWT Story", "");

    Mock::given(method("POST"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&epic_response))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&story_response))
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
      name: "Auth Hardening"
  - action: create
    entity: story
    fields:
      name: "JWT Story"
      epic_id: $ref(my-epic)
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");
}

#[tokio::test]
async fn run_update_story() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let response = full_story_json(42, "Updated Story", "");

    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response))
        .expect(1)
        .mount(&server)
        .await;

    let yaml = r#"
version: 1
operations:
  - action: update
    entity: story
    id: 42
    fields:
      name: "Updated Story"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");
}

#[tokio::test]
async fn run_delete_story() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("DELETE"))
        .and(path("/api/v3/stories/99"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let yaml = r#"
version: 1
operations:
  - action: delete
    entity: story
    id: 99
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");
}

#[tokio::test]
async fn run_comment_on_story() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let response = serde_json::json!({
        "id": 1,
        "text": "A comment",
        "entity_type": "story-comment"
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/stories/500/comments"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&response))
        .expect(1)
        .mount(&server)
        .await;

    let yaml = r#"
version: 1
operations:
  - action: comment
    entity: story
    id: 500
    fields:
      text: "A comment"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");
}

#[tokio::test]
async fn run_link_stories() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let response = serde_json::json!({
        "id": 1,
        "subject_id": 100,
        "object_id": 200,
        "verb": "blocks",
        "entity_type": "story-link"
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/story-links"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&response))
        .expect(1)
        .mount(&server)
        .await;

    let yaml = r#"
version: 1
operations:
  - action: link
    entity: story_link
    fields:
      subject_id: 100
      object_id: 200
      verb: blocks
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");
}

#[tokio::test]
async fn run_repeat_creates_multiple() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Expect 3 story creations
    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(full_story_json(
            300,
            "Bulk Story",
            "",
        )))
        .expect(3)
        .mount(&server)
        .await;

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    repeat:
      - { name: "Story A" }
      - { name: "Story B" }
      - { name: "Story C" }
    fields:
      description: "Shared description"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");
}

#[tokio::test]
async fn run_var_override() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let response = created_response(10, "Sprint 25", "iteration");

    Mock::given(method("POST"))
        .and(path("/api/v3/iterations"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&response))
        .expect(1)
        .mount(&server)
        .await;

    let yaml = r#"
version: 1
vars:
  sprint: "Sprint 24"
  start: "2026-03-02"
  end: "2026-03-15"
operations:
  - action: create
    entity: iteration
    fields:
      name: "$var(sprint)"
      start_date: "$var(start)"
      end_date: "$var(end)"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args_with_vars(&file, vec![("sprint".to_string(), "Sprint 25".to_string())]);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");
}

#[tokio::test]
async fn run_fail_fast_stops_on_error() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // First operation succeeds
    Mock::given(method("POST"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(201).set_body_json(created_response(1, "Epic", "epic")))
        .expect(1)
        .mount(&server)
        .await;

    // Second operation fails
    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(422).set_body_string("name is required"))
        .expect(1)
        .mount(&server)
        .await;

    // Third operation should NOT be called (fail-fast)
    Mock::given(method("POST"))
        .and(path("/api/v3/labels"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(created_response(1, "Label", "label")),
        )
        .expect(0)
        .mount(&server)
        .await;

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: epic
    fields:
      name: "Epic"
  - action: create
    entity: story
    fields:
      name: "Story"
  - action: create
    entity: label
    fields:
      name: "Label"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err(), "Expected error due to fail-fast");
}

#[tokio::test]
async fn run_on_error_continue() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // First operation fails
    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(422).set_body_string("name is required"))
        .expect(1)
        .mount(&server)
        .await;

    // Second operation should still execute
    Mock::given(method("POST"))
        .and(path("/api/v3/labels"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(created_response(1, "Label", "label")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let yaml = r#"
version: 1
on_error: continue
operations:
  - action: create
    entity: story
    fields:
      name: "Story"
  - action: create
    entity: label
    fields:
      name: "Label"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    // Should return error because 1 op failed, but both ops ran
    assert!(result.is_err());
}

#[tokio::test]
async fn run_dry_run_no_api_calls() {
    let (out, buf) = make_dry_run_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // No mocks mounted — dry-run should never call the API
    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    fields:
      name: "Dry Run Story"
  - action: create
    entity: epic
    fields:
      name: "Dry Run Epic"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(output.contains("[1/2] POST /api/v3/stories"));
    assert!(output.contains("[2/2] POST /api/v3/epics"));
    assert!(output.contains("\"name\": \"Dry Run Story\""));
    assert!(output.contains("\"name\": \"Dry Run Epic\""));
}

#[tokio::test]
async fn run_json_output() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Json, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let response = created_response(42, "Test Label", "label");

    Mock::given(method("POST"))
        .and(path("/api/v3/labels"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&response))
        .expect(1)
        .mount(&server)
        .await;

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: label
    fields:
      name: "Test Label"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    // Should contain JSON with operations and summary
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["summary"]["total"], 1);
    assert_eq!(parsed["summary"]["succeeded"], 1);
    assert_eq!(parsed["summary"]["failed"], 0);
    assert_eq!(parsed["operations"][0]["action"], "create");
    assert_eq!(parsed["operations"][0]["entity"], "label");
    assert_eq!(parsed["operations"][0]["status"], "success");
}

#[tokio::test]
async fn run_validation_errors_prevent_execution() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // No mocks — should never reach API
    let yaml = r#"
version: 2
operations:
  - action: create
    entity: story
    fields:
      name: "Test"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("validation error"));
}

#[tokio::test]
async fn run_undeclared_var_override_rejected() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let yaml = r#"
version: 1
vars:
  sprint: "Sprint 24"
operations:
  - action: create
    entity: iteration
    fields:
      name: "$var(sprint)"
      start_date: "2026-01-01"
      end_date: "2026-01-14"
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args_with_vars(&file, vec![("undeclared".to_string(), "value".to_string())]);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not declared"));
}

#[tokio::test]
async fn run_parent_with_children_and_tasks() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Parent story (matched by name)
    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .and(body_partial_json(
            serde_json::json!({"name": "Parent Story"}),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(full_story_json(
            1000,
            "Parent Story",
            "",
        )))
        .expect(1)
        .mount(&server)
        .await;

    // Child Story A (matched by name)
    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .and(body_partial_json(
            serde_json::json!({"name": "Child Story A"}),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(full_story_json(
            1001,
            "Child Story A",
            "Sub-story of parent",
        )))
        .expect(1)
        .mount(&server)
        .await;

    // Child Story B (matched by name)
    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .and(body_partial_json(
            serde_json::json!({"name": "Child Story B"}),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(full_story_json(
            1002,
            "Child Story B",
            "Sub-story of parent",
        )))
        .expect(1)
        .mount(&server)
        .await;

    // Story links (2 link operations)
    Mock::given(method("POST"))
        .and(path("/api/v3/story-links"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 5001,
            "subject_id": 1001,
            "object_id": 1000,
            "verb": "blocks",
            "entity_type": "story-link",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        })))
        .expect(2)
        .mount(&server)
        .await;

    // Tasks on parent story
    Mock::given(method("POST"))
        .and(path("/api/v3/stories/1000/tasks"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(crate::support::task_json(1, 1000, "task", false)),
        )
        .expect(2)
        .mount(&server)
        .await;

    // Tasks on child A
    Mock::given(method("POST"))
        .and(path("/api/v3/stories/1001/tasks"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(crate::support::task_json(2, 1001, "task", false)),
        )
        .expect(2)
        .mount(&server)
        .await;

    // Tasks on child B
    Mock::given(method("POST"))
        .and(path("/api/v3/stories/1002/tasks"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(crate::support::task_json(3, 1002, "task", false)),
        )
        .expect(2)
        .mount(&server)
        .await;

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    alias: parent
    fields:
      name: "Parent Story"

  - action: create
    entity: story
    alias: children
    repeat:
      - { name: "Child Story A" }
      - { name: "Child Story B" }
    fields:
      description: "Sub-story of parent"

  - action: link
    entity: story_link
    fields:
      subject_id: $ref(children.0)
      object_id: $ref(parent)
      verb: blocks

  - action: link
    entity: story_link
    fields:
      subject_id: $ref(children.1)
      object_id: $ref(parent)
      verb: blocks

  - action: create
    entity: task
    repeat:
      - { description: "Parent task 1" }
      - { description: "Parent task 2" }
    fields:
      story_id: $ref(parent)

  - action: create
    entity: task
    repeat:
      - { description: "Child A task 1" }
      - { description: "Child A task 2" }
    fields:
      story_id: $ref(children.0)

  - action: create
    entity: task
    repeat:
      - { description: "Child B task 1" }
      - { description: "Child B task 2" }
    fields:
      story_id: $ref(children.1)
"#;
    let file = write_template(&tmp, yaml);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = run_args(&file);
    let result = template::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");
}
