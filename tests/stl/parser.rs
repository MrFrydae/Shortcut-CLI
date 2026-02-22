use sc::stl::parser::parse;
use sc::stl::types::{Action, Entity, ErrorHandling};

#[test]
fn parse_minimal_template() {
    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    fields:
      name: "My story"
"#;
    let t = parse(yaml).unwrap();
    assert_eq!(t.version, 1);
    assert!(t.meta.is_none());
    assert!(t.vars.is_none());
    assert!(t.on_error.is_none());
    assert_eq!(t.operations.len(), 1);
    assert_eq!(t.operations[0].action, Action::Create);
    assert_eq!(t.operations[0].entity, Entity::Story);
}

#[test]
fn parse_full_template() {
    let yaml = r#"
version: 1
meta:
  description: "Test template"
  author: "@alice"
vars:
  sprint: "Sprint 24"
  team: "@backend"
on_error: continue
operations:
  - action: create
    entity: iteration
    alias: sprint
    fields:
      name: "$var(sprint)"
  - action: create
    entity: epic
    alias: my-epic
    fields:
      name: "Auth Hardening"
  - action: update
    entity: story
    id: 123
    fields:
      state: "Done"
  - action: delete
    entity: story
    id: 456
  - action: comment
    entity: story
    id: 789
    fields:
      text: "A comment"
  - action: link
    entity: story_link
    fields:
      subject_id: 100
      object_id: 200
      verb: blocks
  - action: unlink
    entity: story_link
    id: 50
  - action: check
    entity: task
    id: 99
  - action: uncheck
    entity: task
    id: 99
"#;
    let t = parse(yaml).unwrap();
    assert_eq!(t.version, 1);
    assert_eq!(
        t.meta.as_ref().unwrap().description.as_deref(),
        Some("Test template")
    );
    assert_eq!(t.meta.as_ref().unwrap().author.as_deref(), Some("@alice"));
    assert_eq!(t.vars.as_ref().unwrap().len(), 2);
    assert_eq!(t.on_error, Some(ErrorHandling::Continue));
    assert_eq!(t.operations.len(), 9);

    // Check all action types parsed
    assert_eq!(t.operations[0].action, Action::Create);
    assert_eq!(t.operations[1].action, Action::Create);
    assert_eq!(t.operations[2].action, Action::Update);
    assert_eq!(t.operations[3].action, Action::Delete);
    assert_eq!(t.operations[4].action, Action::Comment);
    assert_eq!(t.operations[5].action, Action::Link);
    assert_eq!(t.operations[6].action, Action::Unlink);
    assert_eq!(t.operations[7].action, Action::Check);
    assert_eq!(t.operations[8].action, Action::Uncheck);

    // Check all entity types parsed
    assert_eq!(t.operations[0].entity, Entity::Iteration);
    assert_eq!(t.operations[1].entity, Entity::Epic);
    assert_eq!(t.operations[2].entity, Entity::Story);
    assert_eq!(t.operations[5].entity, Entity::StoryLink);
    assert_eq!(t.operations[7].entity, Entity::Task);
}

#[test]
fn parse_all_entity_types() {
    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    fields: { name: "s" }
  - action: create
    entity: epic
    fields: { name: "e" }
  - action: create
    entity: iteration
    fields: { name: "i", start_date: "2026-01-01", end_date: "2026-01-14" }
  - action: create
    entity: label
    fields: { name: "l" }
  - action: create
    entity: objective
    fields: { name: "o" }
  - action: create
    entity: milestone
    fields: { name: "m" }
  - action: create
    entity: category
    fields: { name: "c" }
  - action: create
    entity: group
    fields: { name: "g" }
  - action: create
    entity: document
    fields: { name: "d" }
  - action: create
    entity: project
    fields: { name: "p" }
"#;
    let t = parse(yaml).unwrap();
    assert_eq!(t.operations.len(), 10);
    assert_eq!(t.operations[0].entity, Entity::Story);
    assert_eq!(t.operations[1].entity, Entity::Epic);
    assert_eq!(t.operations[2].entity, Entity::Iteration);
    assert_eq!(t.operations[3].entity, Entity::Label);
    assert_eq!(t.operations[4].entity, Entity::Objective);
    assert_eq!(t.operations[5].entity, Entity::Milestone);
    assert_eq!(t.operations[6].entity, Entity::Category);
    assert_eq!(t.operations[7].entity, Entity::Group);
    assert_eq!(t.operations[8].entity, Entity::Document);
    assert_eq!(t.operations[9].entity, Entity::Project);
}

#[test]
fn parse_repeat_block() {
    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    repeat:
      - { name: "Story A", owner: "@alice" }
      - { name: "Story B", owner: "@bob" }
    fields:
      type: feature
      state: "Backlog"
"#;
    let t = parse(yaml).unwrap();
    let op = &t.operations[0];
    assert!(op.repeat.is_some());
    assert_eq!(op.repeat.as_ref().unwrap().len(), 2);
    assert!(op.fields.is_some());
}

#[test]
fn parse_ref_in_id() {
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
    fields:
      name: "A story"
      epic_id: $ref(my-epic)
"#;
    let t = parse(yaml).unwrap();
    assert_eq!(t.operations[0].alias, Some("my-epic".to_string()));
    // The $ref in fields should parse as a string
    let fields = t.operations[1].fields.as_ref().unwrap();
    let epic_id_val = fields
        .get(serde_yaml::Value::String("epic_id".into()))
        .unwrap();
    assert_eq!(
        epic_id_val,
        &serde_yaml::Value::String("$ref(my-epic)".into())
    );
}

#[test]
fn parse_var_in_fields() {
    let yaml = r#"
version: 1
vars:
  sprint: "Sprint 24"
operations:
  - action: create
    entity: iteration
    fields:
      name: "$var(sprint)"
      description: "Focus areas for $var(sprint)"
"#;
    let t = parse(yaml).unwrap();
    let vars = t.vars.as_ref().unwrap();
    assert_eq!(
        vars.get("sprint").unwrap(),
        &serde_yaml::Value::String("Sprint 24".into())
    );
}

#[test]
fn parse_various_field_types() {
    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    fields:
      name: "Test"
      estimate: 5
      archived: false
      epic_id: null
      labels: [bug, frontend]
"#;
    let t = parse(yaml).unwrap();
    let fields = t.operations[0].fields.as_ref().unwrap();
    // Check different value types parsed
    assert!(
        fields
            .get(serde_yaml::Value::String("name".into()))
            .is_some()
    );
    assert!(
        fields
            .get(serde_yaml::Value::String("estimate".into()))
            .is_some()
    );
    assert!(
        fields
            .get(serde_yaml::Value::String("archived".into()))
            .is_some()
    );
    assert!(
        fields
            .get(serde_yaml::Value::String("epic_id".into()))
            .is_some()
    );
    assert!(
        fields
            .get(serde_yaml::Value::String("labels".into()))
            .is_some()
    );
}

#[test]
fn parse_invalid_yaml() {
    let yaml = "not: valid: yaml: [";
    assert!(parse(yaml).is_err());
}

#[test]
fn parse_missing_required_version() {
    let yaml = r#"
operations:
  - action: create
    entity: story
    fields:
      name: "test"
"#;
    assert!(parse(yaml).is_err());
}

#[test]
fn parse_missing_required_operations() {
    let yaml = "version: 1\n";
    assert!(parse(yaml).is_err());
}

#[test]
fn parse_unknown_action() {
    let yaml = r#"
version: 1
operations:
  - action: deploy
    entity: story
    fields:
      name: "test"
"#;
    assert!(parse(yaml).is_err());
}

#[test]
fn parse_unknown_entity() {
    let yaml = r#"
version: 1
operations:
  - action: create
    entity: spaceship
    fields:
      name: "test"
"#;
    assert!(parse(yaml).is_err());
}

#[test]
fn parse_operation_level_on_error() {
    let yaml = r#"
version: 1
operations:
  - action: create
    entity: story
    on_error: continue
    fields:
      name: "Can fail"
"#;
    let t = parse(yaml).unwrap();
    assert_eq!(t.operations[0].on_error, Some(ErrorHandling::Continue));
}

#[test]
fn parse_numeric_id() {
    let yaml = r#"
version: 1
operations:
  - action: update
    entity: story
    id: 42
    fields:
      state: "Done"
"#;
    let t = parse(yaml).unwrap();
    let id = t.operations[0].id.as_ref().unwrap();
    assert_eq!(id.as_u64(), Some(42));
}

#[test]
fn parse_ref_as_id() {
    let yaml = r#"
version: 1
operations:
  - action: create
    entity: epic
    alias: e
    fields:
      name: "Epic"
  - action: update
    entity: epic
    id: $ref(e)
    fields:
      description: "Updated"
"#;
    let t = parse(yaml).unwrap();
    let id = t.operations[1].id.as_ref().unwrap();
    assert_eq!(id.as_str(), Some("$ref(e)"));
}
