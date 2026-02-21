use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use crate::api;

pub fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn cache_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("label_cache.json")
}

pub fn read_cache(cache_dir: &Path) -> Option<HashMap<String, i64>> {
    let path = cache_path(cache_dir);
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn write_cache(map: &HashMap<String, i64>, cache_dir: &Path) {
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

/// Update the label cache from a slice of labels (used by list command).
pub fn update_cache_from_labels(labels: &[api::types::Label], cache_dir: &Path) {
    let map: HashMap<String, i64> = labels
        .iter()
        .map(|l| (normalize_name(&l.name), l.id))
        .collect();
    write_cache(&map, cache_dir);
}

/// Resolve a label name to its ID, using the cache or fetching from the API.
pub async fn resolve_label_id(
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
    let labels = client
        .list_labels()
        .slim(true)
        .send()
        .await
        .map_err(|e| format!("Failed to list labels: {e}"))?;

    let map: HashMap<String, i64> = labels
        .iter()
        .map(|l| (normalize_name(&l.name), l.id))
        .collect();

    write_cache(&map, cache_dir);

    if let Some(&id) = map.get(&normalized) {
        return Ok(id);
    }

    let available: Vec<&str> = labels.iter().map(|l| l.name.as_str()).collect();
    Err(format!(
        "Unknown label '{value}'. Available labels: {}",
        available.join(", ")
    )
    .into())
}
