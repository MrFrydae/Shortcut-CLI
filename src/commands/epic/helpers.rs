use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use crate::api;
use crate::commands::member;

pub async fn resolve_owners(
    owners: &[String],
    client: &api::Client,
    cache_dir: &Path,
) -> Result<Vec<uuid::Uuid>, Box<dyn Error>> {
    let mut ids = Vec::with_capacity(owners.len());
    for owner in owners {
        let uuid = member::resolve_member_id(owner, client, cache_dir).await?;
        ids.push(uuid);
    }
    Ok(ids)
}

fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub async fn resolve_epic_state_id(
    value: &str,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<i64, Box<dyn Error>> {
    // If it parses as i64, use it directly
    if let Ok(id) = value.parse::<i64>() {
        return Ok(id);
    }

    let normalized = normalize_name(value);

    // Try cache first
    if let Some(cache) = read_cache(cache_dir)
        && let Some(&id) = cache.get(&normalized)
    {
        return Ok(id);
    }

    // Cache miss — fetch from API and update cache
    let workflow = client
        .get_epic_workflow()
        .send()
        .await
        .map_err(|e| format!("Failed to fetch epic workflow: {e}"))?;

    let map: HashMap<String, i64> = workflow
        .epic_states
        .iter()
        .map(|s| (normalize_name(&s.name), s.id))
        .collect();

    write_cache(&map, cache_dir);

    if let Some(&id) = map.get(&normalized) {
        return Ok(id);
    }

    let available: Vec<&str> = workflow
        .epic_states
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    Err(format!(
        "Unknown epic state '{value}'. Available states: {}",
        available.join(", ")
    )
    .into())
}

// --- Fetch epic state names for wizard ---

pub async fn fetch_epic_state_names(
    client: &api::Client,
    cache_dir: &Path,
) -> Result<Vec<String>, Box<dyn Error>> {
    let workflow = client
        .get_epic_workflow()
        .send()
        .await
        .map_err(|e| format!("Failed to fetch epic workflow: {e}"))?;

    let map: HashMap<String, i64> = workflow
        .epic_states
        .iter()
        .map(|s| (normalize_name(&s.name), s.id))
        .collect();
    write_cache(&map, cache_dir);

    Ok(workflow
        .epic_states
        .iter()
        .map(|s| s.name.clone())
        .collect())
}

/// Reverse-lookup: given a state ID, return its human-readable name.
pub async fn resolve_epic_state_name(
    state_id: i64,
    client: &api::Client,
    cache_dir: &Path,
) -> String {
    // Check cache for a matching entry (name -> id)
    if let Some(cache) = read_cache(cache_dir) {
        for (name, &id) in &cache {
            if id == state_id {
                return name.clone();
            }
        }
    }

    // Cache miss — fetch from API
    if let Ok(workflow) = client.get_epic_workflow().send().await {
        let map: HashMap<String, i64> = workflow
            .epic_states
            .iter()
            .map(|s| (normalize_name(&s.name), s.id))
            .collect();
        write_cache(&map, cache_dir);

        for s in &workflow.epic_states {
            if s.id == state_id {
                return s.name.clone();
            }
        }
    }

    state_id.to_string()
}

/// Best-effort reverse lookup of a member UUID from the member cache.
pub fn resolve_member_name(uuid: &uuid::Uuid, cache_dir: &Path) -> String {
    let cache_path = cache_dir.join("member_cache.json");
    if let Ok(data) = std::fs::read_to_string(&cache_path)
        && let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&data)
    {
        let uuid_str = uuid.to_string();
        for (mention, id) in &map {
            if id == &uuid_str {
                return format!("@{mention}");
            }
        }
    }
    uuid.to_string()
}

/// Fetch non-archived epics as `IdChoice` items for the story wizard.
pub async fn fetch_epic_choices(
    client: &api::Client,
) -> Result<Vec<crate::interactive::IdChoice>, Box<dyn Error>> {
    let epics = client
        .list_epics()
        .send()
        .await
        .map_err(|e| format!("Failed to list epics: {e}"))?;
    let mut choices: Vec<crate::interactive::IdChoice> = epics
        .iter()
        .filter(|e| !e.archived)
        .map(|e| crate::interactive::IdChoice {
            display: format!("{} (#{})", e.name, e.id),
            id: e.id,
        })
        .collect();
    choices.sort_by(|a, b| a.display.to_lowercase().cmp(&b.display.to_lowercase()));
    Ok(choices)
}

fn cache_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("epic_state_cache.json")
}

fn read_cache(cache_dir: &Path) -> Option<HashMap<String, i64>> {
    let path = cache_path(cache_dir);
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn write_cache(map: &HashMap<String, i64>, cache_dir: &Path) {
    let path = cache_path(cache_dir);

    let Ok(json) = serde_json::to_string_pretty(map) else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let _ = std::fs::write(&path, json);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
}
