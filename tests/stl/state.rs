use std::collections::HashMap;

use shortcut_cli::stl::state::*;

#[test]
fn save_and_load_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("test.state.json");

    let mut state = SyncState::new();
    state.resources.insert(
        "my-epic".to_string(),
        ResourceState::Single {
            entity: "epic".to_string(),
            id: serde_json::json!(55),
            tasks: None,
        },
    );
    state.applied.push("op-2-comment".to_string());

    save_state(&state, &path).unwrap();

    let loaded = load_state(&path).unwrap().expect("state should exist");
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.resources.len(), 1);
    assert_eq!(loaded.applied, vec!["op-2-comment"]);

    match &loaded.resources["my-epic"] {
        ResourceState::Single { entity, id, tasks } => {
            assert_eq!(entity, "epic");
            assert_eq!(*id, serde_json::json!(55));
            assert!(tasks.is_none());
        }
        _ => panic!("Expected Single resource"),
    }
}

#[test]
fn load_missing_file_returns_none() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("nonexistent.state.json");
    let result = load_state(&path).unwrap();
    assert!(result.is_none());
}

#[test]
fn load_corrupt_file_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("bad.state.json");
    std::fs::write(&path, "not valid json").unwrap();
    let result = load_state(&path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to parse"));
}

#[test]
fn load_unsupported_version_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("v2.state.json");
    let state_json = serde_json::json!({
        "version": 2,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
        "resources": {},
        "applied": []
    });
    std::fs::write(&path, serde_json::to_string(&state_json).unwrap()).unwrap();
    let result = load_state(&path);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unsupported state file version")
    );
}

#[test]
fn default_state_path_appends_extension() {
    let result = default_state_path("sprint.shortcut.yml");
    assert_eq!(result.to_str().unwrap(), "sprint.shortcut.yml.state.json");
}

#[test]
fn repeat_resource_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("repeat.state.json");

    let mut entries = HashMap::new();
    entries.insert(
        "auth-design".to_string(),
        EntryState {
            id: serde_json::json!(1001),
            tasks: None,
        },
    );
    entries.insert(
        "auth-impl".to_string(),
        EntryState {
            id: serde_json::json!(1002),
            tasks: None,
        },
    );

    let mut state = SyncState::new();
    state.resources.insert(
        "child-stories".to_string(),
        ResourceState::Repeat {
            entity: "story".to_string(),
            entries,
        },
    );

    save_state(&state, &path).unwrap();
    let loaded = load_state(&path).unwrap().expect("state should exist");

    match &loaded.resources["child-stories"] {
        ResourceState::Repeat { entity, entries } => {
            assert_eq!(entity, "story");
            assert_eq!(entries.len(), 2);
            assert_eq!(entries["auth-design"].id, serde_json::json!(1001));
            assert_eq!(entries["auth-impl"].id, serde_json::json!(1002));
        }
        _ => panic!("Expected Repeat resource"),
    }
}

#[test]
fn single_resource_with_tasks() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("tasks.state.json");

    let mut task_state = HashMap::new();
    task_state.insert(
        "design".to_string(),
        TaskEntry {
            id: serde_json::json!(100),
        },
    );
    task_state.insert(
        "implement".to_string(),
        TaskEntry {
            id: serde_json::json!(101),
        },
    );

    let mut state = SyncState::new();
    state.resources.insert(
        "my-story".to_string(),
        ResourceState::Single {
            entity: "story".to_string(),
            id: serde_json::json!(12345),
            tasks: Some(task_state),
        },
    );

    save_state(&state, &path).unwrap();
    let loaded = load_state(&path).unwrap().expect("state should exist");

    match &loaded.resources["my-story"] {
        ResourceState::Single {
            tasks: Some(tasks), ..
        } => {
            assert_eq!(tasks.len(), 2);
            assert_eq!(tasks["design"].id, serde_json::json!(100));
            assert_eq!(tasks["implement"].id, serde_json::json!(101));
        }
        _ => panic!("Expected Single resource with tasks"),
    }
}
