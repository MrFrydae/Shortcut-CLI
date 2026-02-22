use crate::support::make_output;
use sc::commands::template;

fn validate_args(file: &str) -> template::TemplateArgs {
    template::TemplateArgs {
        action: template::TemplateAction::Validate(template::validate_stl::ValidateArgs {
            file: file.to_string(),
        }),
    }
}

fn write_template(dir: &tempfile::TempDir, yaml: &str) -> String {
    let path = dir.path().join("test.sc.yml");
    std::fs::write(&path, yaml).unwrap();
    path.to_str().unwrap().to_string()
}

#[tokio::test]
async fn validate_valid_template() {
    let out = make_output();
    let tmp = tempfile::tempdir().unwrap();

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    fields:
      name: "Valid Story"
"#;
    let file = write_template(&tmp, yaml);
    let args = validate_args(&file);
    // Validate doesn't need a client or cache_dir
    let result = template::validate_stl::run(
        &template::validate_stl::ValidateArgs { file: file.clone() },
        &out,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn validate_invalid_version() {
    let out = make_output();
    let tmp = tempfile::tempdir().unwrap();

    let yaml = r#"
version: 99
operations:
  - action: create
    entity: story
    fields:
      name: "Test"
"#;
    let file = write_template(&tmp, yaml);
    let result =
        template::validate_stl::run(&template::validate_stl::ValidateArgs { file }, &out).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("validation error"));
}

#[tokio::test]
async fn validate_missing_operations() {
    let out = make_output();
    let tmp = tempfile::tempdir().unwrap();

    let yaml = "version: 1\n";
    let file = write_template(&tmp, yaml);
    let result =
        template::validate_stl::run(&template::validate_stl::ValidateArgs { file }, &out).await;
    // Parse error (missing required field)
    assert!(result.is_err());
}

#[tokio::test]
async fn validate_undeclared_var() {
    let out = make_output();
    let tmp = tempfile::tempdir().unwrap();

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    fields:
      name: "$var(missing)"
"#;
    let file = write_template(&tmp, yaml);
    let result =
        template::validate_stl::run(&template::validate_stl::ValidateArgs { file }, &out).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn validate_undefined_ref() {
    let out = make_output();
    let tmp = tempfile::tempdir().unwrap();

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    fields:
      name: "Test"
      epic_id: $ref(no-such-alias)
"#;
    let file = write_template(&tmp, yaml);
    let result =
        template::validate_stl::run(&template::validate_stl::ValidateArgs { file }, &out).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn validate_missing_required_fields() {
    let out = make_output();
    let tmp = tempfile::tempdir().unwrap();

    let yaml = r#"
version: 1
operations:
  - action: create
    entity: iteration
    fields:
      description: "No name or dates"
"#;
    let file = write_template(&tmp, yaml);
    let result =
        template::validate_stl::run(&template::validate_stl::ValidateArgs { file }, &out).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn validate_file_not_found() {
    let out = make_output();
    let result = template::validate_stl::run(
        &template::validate_stl::ValidateArgs {
            file: "/nonexistent/path.sc.yml".to_string(),
        },
        &out,
    )
    .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to read"));
}

#[tokio::test]
async fn validate_complex_template() {
    let out = make_output();
    let tmp = tempfile::tempdir().unwrap();

    let yaml = r#"
version: 1
vars:
  sprint: "Sprint 24"
  start: "2026-03-02"
  end: "2026-03-15"
operations:
  - action: create
    entity: iteration
    alias: sprint
    fields:
      name: "$var(sprint)"
      start_date: "$var(start)"
      end_date: "$var(end)"
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
      iteration_id: $ref(sprint)
  - action: update
    entity: story
    id: 500
    fields:
      name: "Updated"
  - action: comment
    entity: story
    id: 500
    fields:
      text: "Done in $var(sprint)"
"#;
    let file = write_template(&tmp, yaml);
    let result =
        template::validate_stl::run(&template::validate_stl::ValidateArgs { file }, &out).await;
    assert!(result.is_ok(), "Expected ok, got: {result:?}");
}
