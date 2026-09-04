//! Machine-readable (`--json` / `--toon`) rendering for story commands.
//!
//! The generated API types already derive `Serialize`, so the full story
//! object is emitted as-is. A few fields are then adjusted to keep the
//! shapes that earlier releases exposed and to add resolved names:
//!
//! - `labels` stays a list of label names; the full label objects move to
//!   `label_details`.
//! - `pull_requests[].status` carries the derived `merged` / `closed` /
//!   `draft` / `open` string alongside the raw flags.
//! - `custom_fields[].field_name` is resolved through the custom field cache.
//! - Optional fields the generated types omit when unset are emitted as
//!   `null` / `[]` so every story has the same key set.

use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use crate::api;

use super::super::custom_field;

/// Fields the generated types skip when `None`; emitted as `null` instead.
const NULL_WHEN_MISSING: &[&str] = &[
    "cycle_time",
    "description",
    "formatted_vcs_branch_name",
    "lead_time",
    "parent_story_id",
    "synced_item",
];

/// Fields the generated types skip when empty; emitted as `[]` instead.
const EMPTY_WHEN_MISSING: &[&str] = &["custom_fields", "sub_task_story_ids"];

/// Render a single story (full `Story` or `StorySlim`) for machine-readable output.
pub async fn story_value<T: Serialize>(
    story: &T,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<Value, Box<dyn Error>> {
    let mut values = story_values(std::slice::from_ref(story), client, cache_dir).await?;
    values.pop().ok_or_else(|| "no story to render".into())
}

/// Render a list of stories for machine-readable output.
///
/// Custom field names are resolved with a single lookup across all stories,
/// and no lookup happens when none of the stories carry custom fields.
pub async fn story_values<T: Serialize>(
    stories: &[T],
    client: &api::Client,
    cache_dir: &Path,
) -> Result<Vec<Value>, Box<dyn Error>> {
    let mut values = stories
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;

    let field_ids = collect_custom_field_ids(&values);
    let names = custom_field::resolve_custom_field_names(&field_ids, client, cache_dir).await?;

    for value in &mut values {
        enrich_story(value, &names);
    }
    Ok(values)
}

/// Derive a single status word from a pull request's state flags.
pub fn pr_status(merged: bool, closed: bool, draft: bool) -> &'static str {
    if merged {
        "merged"
    } else if closed {
        "closed"
    } else if draft {
        "draft"
    } else {
        "open"
    }
}

fn collect_custom_field_ids(values: &[Value]) -> Vec<uuid::Uuid> {
    let mut ids: Vec<uuid::Uuid> = values
        .iter()
        .filter_map(|v| v.get("custom_fields").and_then(Value::as_array))
        .flatten()
        .filter_map(|cf| cf.get("field_id").and_then(Value::as_str))
        .filter_map(|s| s.parse().ok())
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

fn enrich_story(value: &mut Value, field_names: &HashMap<String, String>) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };

    for key in NULL_WHEN_MISSING {
        obj.entry(*key).or_insert(Value::Null);
    }
    for key in EMPTY_WHEN_MISSING {
        obj.entry(*key).or_insert_with(|| Value::Array(Vec::new()));
    }

    if let Some(labels) = obj.remove("labels") {
        let names: Vec<Value> = labels
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|l| l.get("name").cloned())
            .collect();
        obj.insert("labels".into(), Value::Array(names));
        obj.insert("label_details".into(), labels);
    }

    if let Some(prs) = obj.get_mut("pull_requests").and_then(Value::as_array_mut) {
        for pr in prs.iter_mut().filter_map(Value::as_object_mut) {
            let flag = |key: &str| pr.get(key).and_then(Value::as_bool).unwrap_or(false);
            let status = pr_status(flag("merged"), flag("closed"), flag("draft"));
            pr.insert("status".into(), Value::from(status));
        }
    }

    if let Some(cfs) = obj.get_mut("custom_fields").and_then(Value::as_array_mut) {
        for cf in cfs.iter_mut().filter_map(Value::as_object_mut) {
            let name = cf
                .get("field_id")
                .and_then(Value::as_str)
                .and_then(|id| field_names.get(id))
                .map(|n| Value::from(n.as_str()))
                .unwrap_or(Value::Null);
            cf.insert("field_name".into(), name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pr_status_precedence() {
        assert_eq!(pr_status(true, true, true), "merged");
        assert_eq!(pr_status(false, true, true), "closed");
        assert_eq!(pr_status(false, false, true), "draft");
        assert_eq!(pr_status(false, false, false), "open");
    }

    #[test]
    fn enrich_keeps_label_names_and_adds_details() {
        let mut value = serde_json::json!({
            "labels": [{"id": 1, "name": "bug"}, {"id": 2, "name": "urgent"}],
        });
        enrich_story(&mut value, &HashMap::new());
        assert_eq!(value["labels"], serde_json::json!(["bug", "urgent"]));
        assert_eq!(value["label_details"][1]["id"], 2);
    }

    #[test]
    fn enrich_adds_pr_status_and_custom_field_name() {
        let mut value = serde_json::json!({
            "pull_requests": [{"number": 7, "merged": false, "closed": false, "draft": true}],
            "custom_fields": [
                {"field_id": "11111111-1111-1111-1111-111111111111", "value_id": "v", "value": "High"},
                {"field_id": "22222222-2222-2222-2222-222222222222", "value_id": "v", "value": "x"},
            ],
        });
        let names: HashMap<String, String> = [(
            "11111111-1111-1111-1111-111111111111".to_string(),
            "Priority".to_string(),
        )]
        .into();
        enrich_story(&mut value, &names);
        assert_eq!(value["pull_requests"][0]["status"], "draft");
        assert_eq!(value["pull_requests"][0]["number"], 7);
        assert_eq!(value["custom_fields"][0]["field_name"], "Priority");
        assert_eq!(value["custom_fields"][0]["value"], "High");
        assert!(value["custom_fields"][1]["field_name"].is_null());
    }

    #[test]
    fn enrich_fills_in_skipped_optional_fields() {
        let mut value = serde_json::json!({"id": 1, "parent_story_id": 5});
        enrich_story(&mut value, &HashMap::new());
        assert_eq!(value["parent_story_id"], 5);
        assert!(value["cycle_time"].is_null());
        assert!(value["synced_item"].is_null());
        assert_eq!(value["custom_fields"], serde_json::json!([]));
        assert_eq!(value["sub_task_story_ids"], serde_json::json!([]));
    }

    #[test]
    fn collect_custom_field_ids_dedups_across_stories() {
        let id = "11111111-1111-1111-1111-111111111111";
        let values = vec![
            serde_json::json!({"custom_fields": [{"field_id": id}]}),
            serde_json::json!({"custom_fields": [{"field_id": id}, {"field_id": "not-a-uuid"}]}),
            serde_json::json!({}),
        ];
        let ids = collect_custom_field_ids(&values);
        assert_eq!(ids, vec![id.parse::<uuid::Uuid>().unwrap()]);
    }
}
