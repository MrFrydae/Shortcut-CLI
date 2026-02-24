use std::collections::HashMap;

use shortcut_cli::stl::parser;
use shortcut_cli::stl::reconciler::{SyncAction, reconcile};
use shortcut_cli::stl::state::*;

fn parse_ops(yaml: &str) -> shortcut_cli::stl::types::Template {
    parser::parse(yaml).unwrap()
}

// ── First run (no state) ──

#[test]
fn first_run_all_creates() {
    let template = parse_ops(
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
    alias: my-story
    fields:
      name: "Story"
"#,
    );
    let actions = reconcile(&template.operations, &None).unwrap();
    assert_eq!(actions.len(), 2);
    assert!(matches!(&actions[0], SyncAction::Create { alias, .. } if alias == "my-epic"));
    assert!(matches!(&actions[1], SyncAction::Create { alias, .. } if alias == "my-story"));
}

#[test]
fn first_run_repeat_creates_entries() {
    let template = parse_ops(
        r#"
version: 1
operations:
  - action: create
    entity: story
    alias: stories
    repeat:
      - key: auth
        name: "Auth"
      - key: db
        name: "Database"
    fields:
      description: "Shared"
"#,
    );
    let actions = reconcile(&template.operations, &None).unwrap();
    assert_eq!(actions.len(), 2);
    assert!(matches!(&actions[0], SyncAction::CreateEntry { key, .. } if key == "auth"));
    assert!(matches!(&actions[1], SyncAction::CreateEntry { key, .. } if key == "db"));
}

// ── Second run (state exists) ──

#[test]
fn second_run_all_updates() {
    let template = parse_ops(
        r#"
version: 1
operations:
  - action: create
    entity: epic
    alias: my-epic
    fields:
      name: "Epic Updated"
"#,
    );

    let mut state = SyncState::new();
    state.resources.insert(
        "my-epic".to_string(),
        ResourceState::Single {
            entity: "epic".to_string(),
            id: serde_json::json!(55),
            tasks: None,
        },
    );

    let actions = reconcile(&template.operations, &Some(state)).unwrap();
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        &actions[0],
        SyncAction::Update { alias, existing_id, .. }
            if alias == "my-epic" && *existing_id == serde_json::json!(55)
    ));
}

#[test]
fn new_alias_creates_while_existing_updates() {
    let template = parse_ops(
        r#"
version: 1
operations:
  - action: create
    entity: epic
    alias: old-epic
    fields:
      name: "Old Epic"
  - action: create
    entity: story
    alias: new-story
    fields:
      name: "New Story"
"#,
    );

    let mut state = SyncState::new();
    state.resources.insert(
        "old-epic".to_string(),
        ResourceState::Single {
            entity: "epic".to_string(),
            id: serde_json::json!(10),
            tasks: None,
        },
    );

    let actions = reconcile(&template.operations, &Some(state)).unwrap();
    assert_eq!(actions.len(), 2);
    assert!(matches!(&actions[0], SyncAction::Update { alias, .. } if alias == "old-epic"));
    assert!(matches!(&actions[1], SyncAction::Create { alias, .. } if alias == "new-story"));
}

#[test]
fn removed_alias_becomes_orphan() {
    let template = parse_ops(
        r#"
version: 1
operations:
  - action: create
    entity: epic
    alias: kept-epic
    fields:
      name: "Kept"
"#,
    );

    let mut state = SyncState::new();
    state.resources.insert(
        "kept-epic".to_string(),
        ResourceState::Single {
            entity: "epic".to_string(),
            id: serde_json::json!(10),
            tasks: None,
        },
    );
    state.resources.insert(
        "removed-story".to_string(),
        ResourceState::Single {
            entity: "story".to_string(),
            id: serde_json::json!(20),
            tasks: None,
        },
    );

    let actions = reconcile(&template.operations, &Some(state)).unwrap();
    assert_eq!(actions.len(), 2);
    assert!(matches!(&actions[0], SyncAction::Update { alias, .. } if alias == "kept-epic"));
    assert!(matches!(
        &actions[1],
        SyncAction::Orphan { alias, entity, ids }
            if alias == "removed-story" && entity == "story" && ids.len() == 1
    ));
}

// ── Repeat reconciliation ──

#[test]
fn repeat_existing_key_updates_new_key_creates() {
    let template = parse_ops(
        r#"
version: 1
operations:
  - action: create
    entity: story
    alias: stories
    repeat:
      - key: existing
        name: "Existing Updated"
      - key: brand-new
        name: "Brand New"
    fields:
      description: "Shared"
"#,
    );

    let mut entries = HashMap::new();
    entries.insert(
        "existing".to_string(),
        EntryState {
            id: serde_json::json!(100),
            tasks: None,
        },
    );

    let mut state = SyncState::new();
    state.resources.insert(
        "stories".to_string(),
        ResourceState::Repeat {
            entity: "story".to_string(),
            entries,
        },
    );

    let actions = reconcile(&template.operations, &Some(state)).unwrap();
    assert_eq!(actions.len(), 2);
    assert!(matches!(
        &actions[0],
        SyncAction::UpdateEntry { key, existing_id, .. }
            if key == "existing" && *existing_id == serde_json::json!(100)
    ));
    assert!(matches!(
        &actions[1],
        SyncAction::CreateEntry { key, .. } if key == "brand-new"
    ));
}

#[test]
fn repeat_removed_key_becomes_orphan_entry() {
    let template = parse_ops(
        r#"
version: 1
operations:
  - action: create
    entity: story
    alias: stories
    repeat:
      - key: kept
        name: "Kept"
    fields:
      description: "Shared"
"#,
    );

    let mut entries = HashMap::new();
    entries.insert(
        "kept".to_string(),
        EntryState {
            id: serde_json::json!(100),
            tasks: None,
        },
    );
    entries.insert(
        "removed".to_string(),
        EntryState {
            id: serde_json::json!(200),
            tasks: None,
        },
    );

    let mut state = SyncState::new();
    state.resources.insert(
        "stories".to_string(),
        ResourceState::Repeat {
            entity: "story".to_string(),
            entries,
        },
    );

    let actions = reconcile(&template.operations, &Some(state)).unwrap();
    assert_eq!(actions.len(), 2);
    assert!(matches!(&actions[0], SyncAction::UpdateEntry { key, .. } if key == "kept"));
    assert!(matches!(
        &actions[1],
        SyncAction::OrphanEntry { key, id, .. }
            if key == "removed" && *id == serde_json::json!(200)
    ));
}

// ── Entity type mismatch ──

#[test]
fn entity_type_mismatch_errors() {
    let template = parse_ops(
        r#"
version: 1
operations:
  - action: create
    entity: story
    alias: my-thing
    fields:
      name: "Thing"
"#,
    );

    let mut state = SyncState::new();
    state.resources.insert(
        "my-thing".to_string(),
        ResourceState::Single {
            entity: "epic".to_string(),
            id: serde_json::json!(10),
            tasks: None,
        },
    );

    let result = reconcile(&template.operations, &Some(state));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("entity type changed"));
}

// ── Side effects ──

#[test]
fn side_effect_not_applied_runs() {
    let template = parse_ops(
        r#"
version: 1
operations:
  - action: comment
    entity: story
    id: 100
    fields:
      text: "Hello"
"#,
    );

    let actions = reconcile(&template.operations, &None).unwrap();
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        &actions[0],
        SyncAction::RunSideEffect { op_index: 0 }
    ));
}

#[test]
fn side_effect_already_applied_skips() {
    let template = parse_ops(
        r#"
version: 1
operations:
  - action: comment
    entity: story
    id: 100
    fields:
      text: "Hello"
"#,
    );

    let mut state = SyncState::new();
    state.applied.push("op-0-comment".to_string());

    let actions = reconcile(&template.operations, &Some(state)).unwrap();
    assert_eq!(actions.len(), 1);
    assert!(matches!(&actions[0], SyncAction::Skip { .. }));
}

// ── Passthrough ──

#[test]
fn explicit_update_is_passthrough() {
    let template = parse_ops(
        r#"
version: 1
operations:
  - action: update
    entity: story
    id: 42
    fields:
      name: "Updated"
"#,
    );

    let actions = reconcile(&template.operations, &None).unwrap();
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        &actions[0],
        SyncAction::Passthrough { op_index: 0 }
    ));
}

#[test]
fn explicit_delete_is_passthrough() {
    let template = parse_ops(
        r#"
version: 1
operations:
  - action: delete
    entity: story
    id: 42
"#,
    );

    let actions = reconcile(&template.operations, &None).unwrap();
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        &actions[0],
        SyncAction::Passthrough { op_index: 0 }
    ));
}

// ── Single/Repeat mismatch ──

#[test]
fn single_to_repeat_mismatch_errors() {
    let template = parse_ops(
        r#"
version: 1
operations:
  - action: create
    entity: story
    alias: my-thing
    repeat:
      - key: a
        name: "A"
    fields:
      description: "Shared"
"#,
    );

    let mut state = SyncState::new();
    state.resources.insert(
        "my-thing".to_string(),
        ResourceState::Single {
            entity: "story".to_string(),
            id: serde_json::json!(10),
            tasks: None,
        },
    );

    let result = reconcile(&template.operations, &Some(state));
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("single operation in state but is now repeat")
    );
}
