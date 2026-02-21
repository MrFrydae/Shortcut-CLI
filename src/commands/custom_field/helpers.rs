use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::api;

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomFieldCacheEntry {
    pub field_id: uuid::Uuid,
    pub display_name: String,
    pub values: HashMap<String, CustomFieldValueCacheEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomFieldValueCacheEntry {
    pub value_id: uuid::Uuid,
    pub display_name: String,
}

fn cache_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("custom_field_cache.json")
}

fn read_cache(cache_dir: &Path) -> Option<HashMap<String, CustomFieldCacheEntry>> {
    let path = cache_path(cache_dir);
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn write_cache(map: &HashMap<String, CustomFieldCacheEntry>, cache_dir: &Path) {
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

pub fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn build_cache(fields: &[api::types::CustomField]) -> HashMap<String, CustomFieldCacheEntry> {
    let mut map = HashMap::new();
    for field in fields {
        let field_name: &str = &field.name;
        let values: HashMap<String, CustomFieldValueCacheEntry> = field
            .values
            .iter()
            .map(|v| {
                let val_str: &str = &v.value;
                (
                    normalize_name(val_str),
                    CustomFieldValueCacheEntry {
                        value_id: v.id,
                        display_name: val_str.to_string(),
                    },
                )
            })
            .collect();
        map.insert(
            normalize_name(field_name),
            CustomFieldCacheEntry {
                field_id: field.id,
                display_name: field_name.to_string(),
                values,
            },
        );
    }
    map
}

/// Resolve a custom field name and value name to their UUIDs.
///
/// Returns `(field_id, value_id)`.
pub async fn resolve_custom_field_value(
    field_name: &str,
    value_name: &str,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(uuid::Uuid, uuid::Uuid), Box<dyn std::error::Error>> {
    let norm_field = normalize_name(field_name);
    let norm_value = normalize_name(value_name);

    // Try cache first
    if let Some(cache) = read_cache(cache_dir)
        && let Some(entry) = cache.get(&norm_field)
        && let Some(val) = entry.values.get(&norm_value)
    {
        return Ok((entry.field_id, val.value_id));
    }

    // Cache miss — fetch from API
    let fields = client
        .list_custom_fields()
        .send()
        .await
        .map_err(|e| format!("Failed to list custom fields: {e}"))?;

    let cache = build_cache(&fields);
    write_cache(&cache, cache_dir);

    let entry = cache.get(&norm_field).ok_or_else(|| {
        let available: Vec<&str> = cache.values().map(|e| e.display_name.as_str()).collect();
        format!(
            "Unknown custom field '{field_name}'. Available fields: {}",
            available.join(", ")
        )
    })?;

    let val = entry.values.get(&norm_value).ok_or_else(|| {
        let available: Vec<&str> = entry
            .values
            .values()
            .map(|v| v.display_name.as_str())
            .collect();
        format!(
            "Unknown value '{value_name}' for custom field '{}'. Available values: {}",
            entry.display_name,
            available.join(", ")
        )
    })?;

    Ok((entry.field_id, val.value_id))
}

/// Resolve custom field IDs to display names for story get output.
///
/// Returns a map of `field_id.to_string()` → `display_name`.
pub async fn resolve_custom_field_names(
    field_ids: &[uuid::Uuid],
    client: &api::Client,
    cache_dir: &Path,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    if field_ids.is_empty() {
        return Ok(HashMap::new());
    }

    // Try cache first
    if let Some(cache) = read_cache(cache_dir) {
        let mut result = HashMap::new();
        let mut all_found = true;
        for fid in field_ids {
            let fid_str = fid.to_string();
            if let Some(entry) = cache.values().find(|e| e.field_id.to_string() == fid_str) {
                result.insert(fid_str, entry.display_name.clone());
            } else {
                all_found = false;
                break;
            }
        }
        if all_found {
            return Ok(result);
        }
    }

    // Cache miss — fetch from API
    let fields = client
        .list_custom_fields()
        .send()
        .await
        .map_err(|e| format!("Failed to list custom fields: {e}"))?;

    let cache = build_cache(&fields);
    write_cache(&cache, cache_dir);

    let mut result = HashMap::new();
    for fid in field_ids {
        let fid_str = fid.to_string();
        if let Some(entry) = cache.values().find(|e| e.field_id.to_string() == fid_str) {
            result.insert(fid_str, entry.display_name.clone());
        } else {
            result.insert(fid_str.clone(), fid_str);
        }
    }

    Ok(result)
}
