use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

use crate::api;

#[derive(Args)]
pub struct CustomFieldArgs {
    #[command(subcommand)]
    pub action: CustomFieldAction,
}

#[derive(Subcommand)]
pub enum CustomFieldAction {
    /// List all custom fields
    List,
    /// Get a custom field by ID
    Get {
        /// The UUID of the custom field
        #[arg(long)]
        id: String,
    },
}

pub async fn run(
    args: &CustomFieldArgs,
    client: &api::Client,
    cache_dir: PathBuf,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        CustomFieldAction::List => run_list(client, &cache_dir).await,
        CustomFieldAction::Get { id } => run_get(id, client).await,
    }
}

async fn run_list(client: &api::Client, cache_dir: &Path) -> Result<(), Box<dyn Error>> {
    let fields = client
        .list_custom_fields()
        .send()
        .await
        .map_err(|e| format!("Failed to list custom fields: {e}"))?;

    if fields.is_empty() {
        println!("No custom fields found");
        return Ok(());
    }

    // Update cache while we have the data
    let cache = build_cache(&fields);
    write_cache(&cache, cache_dir);

    for field in fields.iter() {
        let values: Vec<&str> = field
            .values
            .iter()
            .map(|v| v.value.as_str() as &str)
            .collect();
        let name: &str = &field.name;
        println!("{} - {} ({})", field.id, name, field.field_type);
        if !values.is_empty() {
            println!("  Values: {}", values.join(", "));
        }
    }

    Ok(())
}

async fn run_get(id: &str, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let uuid: uuid::Uuid = id.parse().map_err(|e| format!("Invalid UUID: {e}"))?;

    let field = client
        .get_custom_field()
        .custom_field_public_id(uuid)
        .send()
        .await
        .map_err(|e| format!("Failed to get custom field: {e}"))?;

    let name: &str = &field.name;
    println!("{} - {}", field.id, name);
    println!("  Type:        {}", field.field_type);
    println!("  Enabled:     {}", field.enabled);
    if let Some(desc) = &field.description {
        let desc: &str = desc;
        println!("  Description: {desc}");
    }
    if !field.values.is_empty() {
        println!("  Values:");
        for val in &field.values {
            let color = val
                .color_key
                .as_deref()
                .map(|c| format!(" ({c})"))
                .unwrap_or_default();
            let val_name: &str = &val.value;
            println!("    {} - {}{}", val.id, val_name, color);
        }
    }

    Ok(())
}

// --- Cache types ---

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

// --- Cache I/O ---

fn cache_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("custom_field_cache.json")
}

fn read_cache(cache_dir: &Path) -> Option<HashMap<String, CustomFieldCacheEntry>> {
    let path = cache_path(cache_dir);
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn write_cache(map: &HashMap<String, CustomFieldCacheEntry>, cache_dir: &Path) {
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

fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn build_cache(fields: &[api::types::CustomField]) -> HashMap<String, CustomFieldCacheEntry> {
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
) -> Result<(uuid::Uuid, uuid::Uuid), Box<dyn Error>> {
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
) -> Result<HashMap<String, String>, Box<dyn Error>> {
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
