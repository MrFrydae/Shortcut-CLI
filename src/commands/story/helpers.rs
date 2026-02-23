use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use crate::api;

pub const STORY_TYPES: &[&str] = &["feature", "bug", "chore"];

use super::super::custom_field;
use super::super::member;

// --- Custom field argument parsing ---

pub fn parse_custom_field_arg(arg: &str) -> Result<(&str, &str), Box<dyn Error>> {
    let (name, value) = arg.split_once('=').ok_or_else(|| {
        format!("Invalid custom field format '{arg}': expected 'FieldName=Value'")
    })?;
    let name = name.trim();
    let value = value.trim();
    if name.is_empty() || value.is_empty() {
        return Err(format!(
            "Invalid custom field format '{arg}': name and value must not be empty"
        )
        .into());
    }
    Ok((name, value))
}

pub async fn resolve_custom_field_args(
    args: &[String],
    client: &api::Client,
    cache_dir: &Path,
) -> Result<Vec<api::types::CustomFieldValueParams>, Box<dyn Error>> {
    let mut params = Vec::with_capacity(args.len());
    for arg in args {
        let (field_name, value_name) = parse_custom_field_arg(arg)?;
        let (field_id, value_id) =
            custom_field::resolve_custom_field_value(field_name, value_name, client, cache_dir)
                .await?;
        params.push(api::types::CustomFieldValueParams {
            field_id,
            value: None,
            value_id,
        });
    }
    Ok(params)
}

// --- Owner resolution ---

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

// --- Name normalization ---

pub fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// --- Workflow state resolution ---

pub async fn resolve_workflow_state_id(
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
    let workflows = client.list_workflows().send().await.map_err(|e| {
        format!(
            "Failed to list workflows: {}",
            crate::api::format_api_error(&e)
        )
    })?;

    let mut map: HashMap<String, Vec<(i64, &str)>> = HashMap::new();
    for wf in workflows.iter() {
        for state in &wf.states {
            map.entry(normalize_name(&state.name))
                .or_default()
                .push((state.id, &state.name));
        }
    }

    // Check for ambiguous matches
    let mut cache_map: HashMap<String, i64> = HashMap::new();
    let mut all_names: Vec<String> = Vec::new();
    for wf in workflows.iter() {
        for state in &wf.states {
            all_names.push(state.name.clone());
            let norm = normalize_name(&state.name);
            if let Some(entries) = map.get(&norm)
                && entries.len() == 1
            {
                cache_map.insert(norm, entries[0].0);
            }
        }
    }

    // Check if our target is ambiguous
    if let Some(entries) = map.get(&normalized)
        && entries.len() > 1
    {
        return Err(format!(
            "Ambiguous workflow state '{value}': found in {} workflows. Use a numeric state ID instead.",
            entries.len()
        )
        .into());
    }

    write_cache(&cache_map, cache_dir);

    if let Some(&id) = cache_map.get(&normalized) {
        return Ok(id);
    }

    all_names.sort();
    all_names.dedup();
    Err(format!(
        "Unknown workflow state '{value}'. Available states: {}",
        all_names.join(", ")
    )
    .into())
}

// --- Reverse lookup: state ID → state name ---

pub async fn build_workflow_state_id_map(
    client: &api::Client,
    cache_dir: &Path,
) -> Result<HashMap<i64, String>, Box<dyn Error>> {
    let workflows = client.list_workflows().send().await.map_err(|e| {
        format!(
            "Failed to list workflows: {}",
            crate::api::format_api_error(&e)
        )
    })?;

    let mut map = HashMap::new();
    let mut name_cache: HashMap<String, i64> = HashMap::new();
    for wf in workflows.iter() {
        for state in &wf.states {
            map.insert(state.id, state.name.clone());
            name_cache
                .entry(normalize_name(&state.name))
                .or_insert(state.id);
        }
    }
    write_cache(&name_cache, cache_dir);
    Ok(map)
}

// --- Fetch workflow state names for wizard ---

pub async fn fetch_workflow_state_names(
    client: &api::Client,
    cache_dir: &Path,
) -> Result<Vec<String>, Box<dyn Error>> {
    let workflows = client.list_workflows().send().await.map_err(|e| {
        format!(
            "Failed to list workflows: {}",
            crate::api::format_api_error(&e)
        )
    })?;

    let mut cache_map: HashMap<String, i64> = HashMap::new();
    let mut names: Vec<String> = Vec::new();
    for wf in workflows.iter() {
        for state in &wf.states {
            let norm = normalize_name(&state.name);
            cache_map.entry(norm).or_insert(state.id);
            if !names.contains(&state.name) {
                names.push(state.name.clone());
            }
        }
    }
    write_cache(&cache_map, cache_dir);
    Ok(names)
}

// --- Default workflow state resolution ---

pub async fn get_default_workflow_state_id(
    client: &api::Client,
    cache_dir: &Path,
) -> Result<i64, Box<dyn Error>> {
    // Try dedicated cache first
    if let Some(id) = read_default_state_cache(cache_dir) {
        return Ok(id);
    }

    // Cache miss — fetch from API
    let workflows = client.list_workflows().send().await.map_err(|e| {
        format!(
            "Failed to list workflows: {}",
            crate::api::format_api_error(&e)
        )
    })?;

    let first = workflows.first().ok_or("No workflows found in workspace")?;

    let default_id = first.default_state_id;

    // Populate the name cache as a side-effect
    let mut name_cache: HashMap<String, i64> = HashMap::new();
    for wf in workflows.iter() {
        for state in &wf.states {
            name_cache
                .entry(normalize_name(&state.name))
                .or_insert(state.id);
        }
    }
    write_cache(&name_cache, cache_dir);

    // Cache the default state ID
    write_default_state_cache(default_id, cache_dir);

    Ok(default_id)
}

// --- Cache helpers ---

fn cache_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("workflow_state_cache.json")
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

// --- Default state cache helpers ---

fn default_state_cache_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("default_workflow_state_cache.json")
}

fn read_default_state_cache(cache_dir: &Path) -> Option<i64> {
    let path = default_state_cache_path(cache_dir);
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn write_default_state_cache(id: i64, cache_dir: &Path) {
    let path = default_state_cache_path(cache_dir);

    let Ok(json) = serde_json::to_string_pretty(&id) else {
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
