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
