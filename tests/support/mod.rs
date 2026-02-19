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
