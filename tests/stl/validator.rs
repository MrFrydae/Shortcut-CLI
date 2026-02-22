use sc::stl::parser;
use sc::stl::validator::{ValidationError, validate};

fn parse_and_validate(yaml: &str) -> Vec<ValidationError> {
    let template = parser::parse(yaml).unwrap();
    validate(&template)
}

#[test]
fn valid_minimal_template() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: create
    entity: story
    fields:
      name: "Test"
"#,
    );
    assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
}

#[test]
fn invalid_version() {
    let errors = parse_and_validate(
        r#"
version: 2
operations:
  - action: create
    entity: story
    fields:
      name: "Test"
"#,
    );
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("unsupported version 2"));
}

#[test]
fn duplicate_alias() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: create
    entity: story
    alias: my-story
    fields:
      name: "Story 1"
  - action: create
    entity: story
    alias: my-story
    fields:
      name: "Story 2"
"#,
    );
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("duplicate alias 'my-story'"));
}

#[test]
fn undefined_ref() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: create
    entity: story
    fields:
      name: "Test"
      epic_id: $ref(nonexistent)
"#,
    );
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0]
            .message
            .contains("references undefined alias 'nonexistent'")
    );
}

#[test]
fn undeclared_var() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: create
    entity: story
    fields:
      name: "$var(missing_var)"
"#,
    );
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0]
            .message
            .contains("references undeclared variable 'missing_var'")
    );
}

#[test]
fn declared_var_is_ok() {
    let errors = parse_and_validate(
        r#"
version: 1
vars:
  sprint: "Sprint 24"
operations:
  - action: create
    entity: story
    fields:
      name: "$var(sprint)"
"#,
    );
    assert!(errors.is_empty());
}

#[test]
fn missing_id_on_update() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: update
    entity: story
    fields:
      state: "Done"
"#,
    );
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("requires an 'id' field"));
}

#[test]
fn missing_id_on_delete() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: delete
    entity: story
"#,
    );
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("requires an 'id' field"));
}

#[test]
fn missing_required_fields_on_create() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: create
    entity: story
    fields:
      description: "No name"
"#,
    );
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("requires field 'name'"));
}

#[test]
fn missing_multiple_required_fields() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: create
    entity: iteration
    fields:
      description: "No name or dates"
"#,
    );
    assert_eq!(errors.len(), 3);
    let msgs: Vec<&str> = errors.iter().map(|e| e.message.as_str()).collect();
    assert!(msgs.iter().any(|m| m.contains("'name'")));
    assert!(msgs.iter().any(|m| m.contains("'start_date'")));
    assert!(msgs.iter().any(|m| m.contains("'end_date'")));
}

#[test]
fn invalid_action_entity_comment_on_label() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: comment
    entity: label
    id: 1
    fields:
      text: "Hello"
"#,
    );
    assert!(errors.iter().any(|e| {
        e.message
            .contains("'comment' action is not valid for 'label'")
    }));
}

#[test]
fn invalid_alias_name() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: create
    entity: story
    alias: "123invalid"
    fields:
      name: "Test"
"#,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("invalid alias name"))
    );
}

#[test]
fn invalid_var_name() {
    let errors = parse_and_validate(
        r#"
version: 1
vars:
  "123bad": "value"
operations:
  - action: create
    entity: story
    fields:
      name: "Test"
"#,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("invalid variable name"))
    );
}

#[test]
fn unknown_field_for_entity() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: create
    entity: label
    fields:
      name: "Bug"
      warp_drive: true
"#,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("unknown field 'warp_drive'"))
    );
}

#[test]
fn ref_in_id_to_defined_alias() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: create
    entity: epic
    alias: my-epic
    fields:
      name: "Epic"
  - action: update
    entity: epic
    id: $ref(my-epic)
    fields:
      description: "Updated"
"#,
    );
    assert!(errors.is_empty());
}

#[test]
fn ref_with_field_access() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: create
    entity: epic
    alias: my-epic
    fields:
      name: "Epic"
  - action: create
    entity: story
    fields:
      name: "Story"
      description: "See $ref(my-epic.app_url)"
"#,
    );
    assert!(errors.is_empty());
}

#[test]
fn forward_reference_is_error() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: create
    entity: story
    fields:
      name: "Story"
      epic_id: $ref(later-epic)
  - action: create
    entity: epic
    alias: later-epic
    fields:
      name: "Epic"
"#,
    );
    assert!(!errors.is_empty());
    assert!(
        errors[0]
            .message
            .contains("references undefined alias 'later-epic'")
    );
}

#[test]
fn comment_valid_on_story_and_epic() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: comment
    entity: story
    id: 100
    fields:
      text: "Hello"
  - action: comment
    entity: epic
    id: 200
    fields:
      text: "World"
"#,
    );
    assert!(errors.is_empty());
}

#[test]
fn check_uncheck_valid_on_task() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: check
    entity: task
    id: 10
  - action: uncheck
    entity: task
    id: 10
"#,
    );
    assert!(errors.is_empty());
}

#[test]
fn link_valid_on_story_link() {
    let errors = parse_and_validate(
        r#"
version: 1
operations:
  - action: link
    entity: story_link
    fields:
      subject_id: 100
      object_id: 200
      verb: blocks
"#,
    );
    assert!(errors.is_empty());
}
