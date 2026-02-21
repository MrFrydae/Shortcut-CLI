use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use crate::api;
use crate::commands::member;

pub async fn resolve_group_id(
    id_or_mention: &str,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<uuid::Uuid, Box<dyn Error>> {
    if let Some(mention) = id_or_mention.strip_prefix('@') {
        // Try cache first
        if let Some(cache) = read_cache(cache_dir)
            && let Some(uuid_str) = cache.get(mention)
            && let Ok(uuid) = uuid_str.parse::<uuid::Uuid>()
        {
            return Ok(uuid);
        }

        // Cache miss — fetch from API and update cache
        let groups = client
            .list_groups()
            .send()
            .await
            .map_err(|e| format!("Failed to list groups: {e}"))?;

        write_cache(&groups, cache_dir);

        for g in groups.iter() {
            if g.mention_name.as_str() == mention {
                return Ok(g.id);
            }
        }

        Err(format!("No group found with mention name @{mention}").into())
    } else {
        id_or_mention
            .parse::<uuid::Uuid>()
            .map_err(|_| format!("Invalid group ID (expected UUID): {id_or_mention}").into())
    }
}

pub async fn resolve_members(
    members: &[String],
    client: &api::Client,
    cache_dir: &Path,
) -> Result<Vec<uuid::Uuid>, Box<dyn Error>> {
    let mut ids = Vec::with_capacity(members.len());
    for m in members {
        let uuid = member::resolve_member_id(m, client, cache_dir).await?;
        ids.push(uuid);
    }
    Ok(ids)
}

fn cache_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("group_cache.json")
}

fn read_cache(cache_dir: &Path) -> Option<HashMap<String, String>> {
    let path = cache_path(cache_dir);
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn write_cache(
    groups: &progenitor_client::ResponseValue<Vec<api::types::Group>>,
    cache_dir: &Path,
) {
    let path = cache_path(cache_dir);

    let map: HashMap<String, String> = groups
        .iter()
        .map(|g| (g.mention_name.to_string(), g.id.to_string()))
        .collect();

    let Ok(json) = serde_json::to_string_pretty(&map) else {
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
