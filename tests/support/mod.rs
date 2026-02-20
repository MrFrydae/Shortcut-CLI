use std::cell::RefCell;

use sc::auth::{AuthError, TokenStore};

/// In-memory token store for tests.
pub struct MockTokenStore {
    token: RefCell<Option<String>>,
}

impl MockTokenStore {
    pub fn new() -> Self {
        Self {
            token: RefCell::new(None),
        }
    }

    #[allow(dead_code)]
    pub fn with_token(token: &str) -> Self {
        Self {
            token: RefCell::new(Some(token.to_string())),
        }
    }
}

impl TokenStore for MockTokenStore {
    fn store_token(&self, token: &str) -> Result<(), AuthError> {
        *self.token.borrow_mut() = Some(token.to_string());
        Ok(())
    }

    fn get_token(&self) -> Result<String, AuthError> {
        self.token.borrow().clone().ok_or(AuthError::NotFound)
    }

    fn delete_token(&self) -> Result<(), AuthError> {
        *self.token.borrow_mut() = None;
        Ok(())
    }
}

/// Build a JSON value representing a valid `EpicSlim` response object.
///
/// The generated struct has `deny_unknown_fields` and many required fields,
/// so we must provide every single one.
pub fn epic_json(id: i64, name: &str, description: Option<&str>) -> serde_json::Value {
    let stats = serde_json::json!({
        "num_points": 0,
        "num_points_backlog": 0,
        "num_points_done": 0,
        "num_points_started": 0,
        "num_points_unstarted": 0,
        "num_related_documents": 0,
        "num_stories_backlog": 0,
        "num_stories_done": 0,
        "num_stories_started": 0,
        "num_stories_total": 0,
        "num_stories_unestimated": 0,
        "num_stories_unstarted": 0,
        "last_story_update": null
    });

    let mut epic = serde_json::json!({
        "id": id,
        "name": name,
        "archived": false,
        "started": false,
        "completed": false,
        "state": "to do",
        "entity_type": "epic",
        "app_url": format!("https://app.shortcut.com/test/epic/{id}"),
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "started_at": null,
        "started_at_override": null,
        "completed_at": null,
        "completed_at_override": null,
        "deadline": null,
        "planned_start_date": null,
        "milestone_id": null,
        "global_id": format!("global-epic-{id}"),
        "group_id": null,
        "group_ids": [],
        "group_mention_ids": [],
        "label_ids": [],
        "member_mention_ids": [],
        "mention_ids": [],
        "owner_ids": [],
        "follower_ids": [],
        "project_ids": [],
        "objective_ids": [],
        "associated_groups": null,
        "labels": [],
        "position": 1,
        "requested_by_id": "00000000-0000-0000-0000-000000000001",
        "epic_state_id": 1,
        "external_id": null,
        "productboard_id": null,
        "productboard_name": null,
        "productboard_plugin_id": null,
        "productboard_url": null,
        "stories_without_projects": 0,
        "stats": stats
    });

    if let Some(desc) = description {
        epic["description"] = serde_json::Value::String(desc.to_string());
    }

    epic
}

/// Build a JSON value representing a valid full `Epic` response object.
///
/// The full `Epic` type (returned by update/get) requires `description` as a
/// non-optional `String` and includes a `comments` array, unlike `EpicSlim`.
pub fn full_epic_json(id: i64, name: &str, description: &str) -> serde_json::Value {
    let mut epic = epic_json(id, name, Some(description));
    epic["comments"] = serde_json::json!([]);
    epic
}

/// Build a JSON value representing a valid `Workflow` response object.
pub fn workflow_json(id: i64, name: &str, states: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": name,
        "description": "",
        "entity_type": "workflow",
        "project_ids": [],
        "states": states,
        "auto_assign_owner": false,
        "team_id": 1,
        "default_state_id": 100,
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    })
}

/// Build a JSON value representing a valid `WorkflowState` response object.
pub fn workflow_state_json(id: i64, name: &str, type_: &str, position: i64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": name,
        "type": type_,
        "position": position,
        "description": "",
        "entity_type": "workflow-state",
        "global_id": format!("global-ws-{id}"),
        "verb": null,
        "color": "#ffffff",
        "num_stories": 0,
        "num_story_templates": 0,
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    })
}

/// Build a JSON value representing a valid `Profile` for a `Member`.
///
/// When `display_icon` is `None` the field is serialised as JSON `null`,
/// matching what the Shortcut API returns for members without a custom icon.
pub fn profile_json(
    id: &str,
    mention_name: &str,
    name: &str,
    display_icon: Option<serde_json::Value>,
) -> serde_json::Value {
    let icon = display_icon.unwrap_or(serde_json::Value::Null);
    serde_json::json!({
        "deactivated": false,
        "display_icon": icon,
        "email_address": format!("{mention_name}@example.com"),
        "entity_type": "profile",
        "gravatar_hash": null,
        "id": id,
        "is_owner": false,
        "mention_name": mention_name,
        "name": name
    })
}

/// Build a JSON value representing a valid `Member` response object.
///
/// Pass `display_icon: None` to test members whose profile has a null icon.
pub fn member_json(
    id: &str,
    mention_name: &str,
    name: &str,
    role: &str,
    disabled: bool,
    display_icon: Option<serde_json::Value>,
) -> serde_json::Value {
    serde_json::json!({
        "created_at": "2024-01-01T00:00:00Z",
        "created_without_invite": false,
        "disabled": disabled,
        "entity_type": "member",
        "global_id": format!("global-member-{id}"),
        "group_ids": [],
        "id": id,
        "profile": profile_json(id, mention_name, name, display_icon),
        "role": role,
        "state": "full",
        "updated_at": "2024-01-01T00:00:00Z"
    })
}

/// A default `Icon` JSON value for tests that don't care about the icon.
pub fn default_icon() -> serde_json::Value {
    serde_json::json!({
        "created_at": "2024-01-01T00:00:00Z",
        "entity_type": "icon",
        "id": "00000000-0000-0000-0000-000000000099",
        "updated_at": "2024-01-01T00:00:00Z",
        "url": "https://example.com/icon.png"
    })
}

/// Build a JSON value representing a valid `EpicWorkflow` response object.
pub fn epic_workflow_json(states: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "created_at": "2024-01-01T00:00:00Z",
        "default_epic_state_id": 500000010,
        "entity_type": "epic-workflow",
        "epic_states": states,
        "id": 1,
        "updated_at": "2024-01-01T00:00:00Z"
    })
}

/// Build a JSON value representing a valid `EpicState` response object.
pub fn epic_state_json(id: i64, name: &str, type_: &str, position: i64) -> serde_json::Value {
    serde_json::json!({
        "color": "#ffffff",
        "created_at": "2024-01-01T00:00:00Z",
        "description": "",
        "entity_type": "epic-state",
        "global_id": format!("global-es-{id}"),
        "id": id,
        "name": name,
        "position": position,
        "type": type_,
        "updated_at": "2024-01-01T00:00:00Z"
    })
}

/// Build a JSON value representing a valid `MemberInfo` response object.
pub fn member_info_json(name: &str, mention_name: &str) -> serde_json::Value {
    let workspace2 = serde_json::json!({
        "id": "00000000-0000-0000-0000-000000000002",
        "name": "Test Workspace",
        "url_slug": "test-workspace",
        "estimate_scale": [1, 2, 3],
        "created_at": "2024-01-01T00:00:00Z",
        "default_workflow_id": 1,
        "utc_offset": "+00:00"
    });

    serde_json::json!({
        "id": "00000000-0000-0000-0000-000000000001",
        "is_owner": false,
        "mention_name": mention_name,
        "name": name,
        "role": "member",
        "workspace2": workspace2,
        "organization2": {
            "id": "00000000-0000-0000-0000-000000000003"
        }
    })
}

/// Common story fields shared between slim and full variants.
/// Built incrementally to avoid `serde_json::json!` recursion limit.
fn story_base(id: i64, name: &str) -> serde_json::Map<String, serde_json::Value> {
    use serde_json::Value;

    let mut m = serde_json::Map::new();
    m.insert("id".into(), Value::from(id));
    m.insert("name".into(), Value::from(name));
    m.insert("story_type".into(), Value::from("feature"));
    m.insert(
        "app_url".into(),
        Value::from(format!("https://app.shortcut.com/test/story/{id}")),
    );
    m.insert("archived".into(), Value::from(false));
    m.insert("blocked".into(), Value::from(false));
    m.insert("blocker".into(), Value::from(false));
    m.insert("completed".into(), Value::from(false));
    m.insert("completed_at".into(), Value::Null);
    m.insert("completed_at_override".into(), Value::Null);
    m.insert("created_at".into(), Value::from("2024-01-01T00:00:00Z"));
    m.insert("deadline".into(), Value::Null);
    m.insert("entity_type".into(), Value::from("story"));
    m.insert("epic_id".into(), Value::Null);
    m.insert("estimate".into(), Value::Null);
    m.insert("external_id".into(), Value::Null);
    m.insert("external_links".into(), serde_json::json!([]));
    m.insert("follower_ids".into(), serde_json::json!([]));
    m.insert(
        "global_id".into(),
        Value::from(format!("global-story-{id}")),
    );
    m.insert("group_id".into(), Value::Null);
    m.insert("group_mention_ids".into(), serde_json::json!([]));
    m.insert("iteration_id".into(), Value::Null);
    m.insert("label_ids".into(), serde_json::json!([]));
    m.insert("labels".into(), serde_json::json!([]));
    m.insert("member_mention_ids".into(), serde_json::json!([]));
    m.insert("mention_ids".into(), serde_json::json!([]));
    m.insert("moved_at".into(), Value::Null);
    m.insert("owner_ids".into(), serde_json::json!([]));
    m.insert("position".into(), Value::from(1));
    m.insert("previous_iteration_ids".into(), serde_json::json!([]));
    m.insert("project_id".into(), Value::Null);
    m.insert(
        "requested_by_id".into(),
        Value::from("00000000-0000-0000-0000-000000000001"),
    );
    m.insert("started".into(), Value::from(false));
    m.insert("started_at".into(), Value::Null);
    m.insert("started_at_override".into(), Value::Null);
    m.insert(
        "stats".into(),
        serde_json::json!({ "num_related_documents": 0 }),
    );
    m.insert("story_links".into(), serde_json::json!([]));
    m.insert("story_template_id".into(), Value::Null);
    m.insert("updated_at".into(), Value::from("2024-01-01T00:00:00Z"));
    m.insert("workflow_id".into(), Value::from(500000006_i64));
    m.insert("workflow_state_id".into(), Value::from(500000007_i64));
    m
}

/// Build a JSON value representing a valid `StorySlim` response object.
///
/// `description` is optional on the slim variant; pass `None` to omit it.
pub fn story_json(id: i64, name: &str, description: Option<&str>) -> serde_json::Value {
    let mut m = story_base(id, name);
    m.insert("comment_ids".into(), serde_json::json!([]));
    m.insert("file_ids".into(), serde_json::json!([]));
    m.insert("linked_file_ids".into(), serde_json::json!([]));
    m.insert("task_ids".into(), serde_json::json!([]));
    m.insert("num_tasks_completed".into(), serde_json::Value::from(0));
    if let Some(desc) = description {
        m.insert(
            "description".into(),
            serde_json::Value::String(desc.to_string()),
        );
    }
    serde_json::Value::Object(m)
}

/// Build a JSON value representing a valid full `Story` response object.
///
/// The full `Story` type (returned by create/update/get) requires `description`
/// as a non-optional `String` and includes full resource arrays instead of IDs.
pub fn full_story_json(id: i64, name: &str, description: &str) -> serde_json::Value {
    let mut m = story_base(id, name);
    m.insert(
        "description".into(),
        serde_json::Value::String(description.to_string()),
    );
    m.insert("branches".into(), serde_json::json!([]));
    m.insert("comments".into(), serde_json::json!([]));
    m.insert("commits".into(), serde_json::json!([]));
    m.insert("files".into(), serde_json::json!([]));
    m.insert("linked_files".into(), serde_json::json!([]));
    m.insert("pull_requests".into(), serde_json::json!([]));
    m.insert("tasks".into(), serde_json::json!([]));
    serde_json::Value::Object(m)
}

/// Build a full `Story` JSON with a custom tasks array.
pub fn full_story_json_with_tasks(
    id: i64,
    name: &str,
    description: &str,
    tasks: Vec<serde_json::Value>,
) -> serde_json::Value {
    let mut story = full_story_json(id, name, description);
    story["tasks"] = serde_json::Value::Array(tasks);
    story
}

/// Build a full `Story` JSON with a custom story_links array.
pub fn full_story_json_with_links(
    id: i64,
    name: &str,
    description: &str,
    links: Vec<serde_json::Value>,
) -> serde_json::Value {
    let mut story = full_story_json(id, name, description);
    story["story_links"] = serde_json::Value::Array(links);
    story
}

/// Build a JSON value representing a valid `TypedStoryLink` (embedded in Story response).
pub fn typed_story_link_json(
    id: i64,
    subject_id: i64,
    object_id: i64,
    verb: &str,
    type_: &str,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "subject_id": subject_id,
        "object_id": object_id,
        "verb": verb,
        "type": type_,
        "entity_type": "story-link",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "subject_workflow_state_id": 500000007
    })
}

/// Build a JSON value representing a valid `StoryLink` response object (from create/get).
pub fn story_link_json(id: i64, subject_id: i64, object_id: i64, verb: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "subject_id": subject_id,
        "object_id": object_id,
        "verb": verb,
        "entity_type": "story-link",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "subject_workflow_state_id": 500000007
    })
}

/// Build a JSON value representing a valid `Task` response object.
pub fn task_json(id: i64, story_id: i64, description: &str, complete: bool) -> serde_json::Value {
    use serde_json::Value;

    let mut m = serde_json::Map::new();
    m.insert("id".into(), Value::from(id));
    m.insert("story_id".into(), Value::from(story_id));
    m.insert("description".into(), Value::from(description));
    m.insert("complete".into(), Value::from(complete));
    m.insert(
        "completed_at".into(),
        if complete {
            Value::from("2024-01-02T00:00:00Z")
        } else {
            Value::Null
        },
    );
    m.insert("created_at".into(), Value::from("2024-01-01T00:00:00Z"));
    m.insert("entity_type".into(), Value::from("task"));
    m.insert("external_id".into(), Value::Null);
    m.insert("global_id".into(), Value::from(format!("global-task-{id}")));
    m.insert("group_mention_ids".into(), serde_json::json!([]));
    m.insert("member_mention_ids".into(), serde_json::json!([]));
    m.insert("mention_ids".into(), serde_json::json!([]));
    m.insert("owner_ids".into(), serde_json::json!([]));
    m.insert("position".into(), Value::from(1));
    m.insert("updated_at".into(), Value::from("2024-01-01T00:00:00Z"));
    Value::Object(m)
}
