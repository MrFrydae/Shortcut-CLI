use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use clap::Args;

use crate::api;

#[derive(Args)]
pub struct MemberArgs {
    /// List all workspace members
    #[arg(long)]
    pub list: bool,

    /// Get a member by UUID or @mention_name
    #[arg(long)]
    pub id: Option<String>,
}

pub async fn run(
    args: &MemberArgs,
    client: &api::Client,
    cache_dir: Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    if args.list {
        run_list(client, &cache_dir).await
    } else if let Some(id) = &args.id {
        run_get(id, client, &cache_dir).await
    } else {
        Ok(())
    }
}

async fn run_list(client: &api::Client, cache_dir: &Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    let members = client
        .list_members()
        .send()
        .await
        .map_err(|e| format!("Failed to list members: {e}"))?;

    for m in members.iter() {
        println!(
            "{} - @{} - {} ({})",
            m.id,
            m.profile.mention_name,
            m.profile.name.as_deref().unwrap_or(""),
            m.role,
        );
    }

    write_cache(&members, cache_dir);

    Ok(())
}

async fn run_get(
    id_or_mention: &str,
    client: &api::Client,
    cache_dir: &Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let uuid = resolve_member_id(id_or_mention, client, cache_dir).await?;

    let member = client
        .get_member()
        .member_public_id(uuid)
        .send()
        .await
        .map_err(|e| format!("Failed to get member: {e}"))?;

    print_member_detail(&member);
    Ok(())
}

async fn resolve_member_id(
    id_or_mention: &str,
    client: &api::Client,
    cache_dir: &Option<PathBuf>,
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
        let members = client
            .list_members()
            .send()
            .await
            .map_err(|e| format!("Failed to list members: {e}"))?;

        write_cache(&members, cache_dir);

        for m in members.iter() {
            if m.profile.mention_name == mention {
                return Ok(m.id);
            }
        }

        Err(format!("No member found with mention name @{mention}").into())
    } else {
        id_or_mention
            .parse::<uuid::Uuid>()
            .map_err(|_| format!("Invalid member ID: {id_or_mention}").into())
    }
}

fn print_member_detail(member: &api::types::Member) {
    let name = member.profile.name.as_deref().unwrap_or("");
    println!("{name} (@{})", member.profile.mention_name);
    println!("  ID:       {}", member.id);
    println!("  Role:     {}", member.role);
    println!("  State:    {:?}", member.state);
    println!("  Disabled: {}", member.disabled);
    if let Some(email) = &member.profile.email_address {
        println!("  Email:    {email}");
    }
}

// --- Cache helpers ---

fn cache_path(cache_dir: &Option<PathBuf>) -> Option<PathBuf> {
    let dir = match cache_dir {
        Some(d) => d.clone(),
        None => dirs::config_dir()?.join("sc-cli"),
    };
    Some(dir.join("member_cache.json"))
}

fn read_cache(cache_dir: &Option<PathBuf>) -> Option<HashMap<String, String>> {
    let path = cache_path(cache_dir)?;
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn write_cache(
    members: &progenitor_client::ResponseValue<Vec<api::types::Member>>,
    cache_dir: &Option<PathBuf>,
) {
    let Some(path) = cache_path(cache_dir) else {
        return;
    };

    let map: HashMap<String, String> = members
        .iter()
        .map(|m| (m.profile.mention_name.clone(), m.id.to_string()))
        .collect();

    let Ok(json) = serde_json::to_string_pretty(&map) else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let _ = std::fs::write(&path, json);

    // Set 0600 permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
}
