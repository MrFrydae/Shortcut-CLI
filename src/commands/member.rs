use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use clap::Args;

use crate::api;
use crate::out_println;
use crate::output::{OutputConfig, Table};

#[derive(Args)]
pub struct MemberArgs {
    /// List all workspace members
    #[arg(long)]
    pub list: bool,

    /// Get a member by UUID or @mention_name
    #[arg(long)]
    pub id: Option<String>,

    /// Filter by role (admin, member, owner, observer)
    #[arg(long)]
    pub role: Option<String>,

    /// Show only active (non-disabled) members
    #[arg(long)]
    pub active: bool,
}

const VALID_ROLES: &[&str] = &["admin", "member", "owner", "observer"];

pub async fn run(
    args: &MemberArgs,
    client: &api::Client,
    cache_dir: PathBuf,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if args.list {
        if let Some(role) = &args.role {
            let lower = role.to_lowercase();
            if !VALID_ROLES.contains(&lower.as_str()) {
                return Err(format!(
                    "Invalid role '{role}'. Valid roles: {}",
                    VALID_ROLES.join(", ")
                )
                .into());
            }
        }
        run_list(args.role.as_deref(), args.active, client, &cache_dir, out).await
    } else if let Some(id) = &args.id {
        run_get(id, client, &cache_dir, out).await
    } else {
        Ok(())
    }
}

async fn run_list(
    role_filter: Option<&str>,
    active_only: bool,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let members = client
        .list_members()
        .send()
        .await
        .map_err(|e| format!("Failed to list members: {e}"))?;

    if out.is_json() {
        let json: Vec<serde_json::Value> = members
            .iter()
            .filter(|m| !active_only || !m.disabled)
            .filter(|m| role_filter.is_none_or(|r| m.role.eq_ignore_ascii_case(r)))
            .map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "mention_name": m.profile.mention_name,
                    "name": m.profile.name,
                    "role": m.role,
                })
            })
            .collect();
        out_println!(out, "{}", serde_json::to_string_pretty(&json)?);
        write_cache(&members, cache_dir);
        return Ok(());
    }

    if out.is_quiet() {
        for m in members.iter() {
            if active_only && m.disabled {
                continue;
            }
            if let Some(role) = role_filter
                && !m.role.eq_ignore_ascii_case(role)
            {
                continue;
            }
            out_println!(out, "{}", m.id);
        }
        write_cache(&members, cache_dir);
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "@Mention", "Name", "Role"]);
    for m in members.iter() {
        if active_only && m.disabled {
            continue;
        }
        if let Some(role) = role_filter
            && !m.role.eq_ignore_ascii_case(role)
        {
            continue;
        }
        table.add_row(vec![
            m.id.to_string(),
            format!("@{}", m.profile.mention_name),
            m.profile.name.as_deref().unwrap_or("").to_string(),
            m.role.clone(),
        ]);
    }
    out.write_str(format_args!("{}", table.render()))?;

    write_cache(&members, cache_dir);

    Ok(())
}

async fn run_get(
    id_or_mention: &str,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let uuid = resolve_member_id(id_or_mention, client, cache_dir).await?;

    let member = client
        .get_member()
        .member_public_id(uuid)
        .send()
        .await
        .map_err(|e| format!("Failed to get member: {e}"))?;

    if out.is_json() {
        let json = serde_json::json!({
            "id": member.id,
            "mention_name": member.profile.mention_name,
            "name": member.profile.name,
            "role": member.role,
            "disabled": member.disabled,
        });
        out_println!(out, "{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }
    if out.is_quiet() {
        out_println!(out, "{}", member.id);
        return Ok(());
    }

    let name = member.profile.name.as_deref().unwrap_or("");
    out_println!(out, "{name} (@{})", member.profile.mention_name);
    out_println!(out, "  ID:       {}", member.id);
    out_println!(out, "  Role:     {}", member.role);
    out_println!(out, "  State:    {:?}", member.state);
    out_println!(out, "  Disabled: {}", member.disabled);
    if let Some(email) = &member.profile.email_address {
        out_println!(out, "  Email:    {email}");
    }
    Ok(())
}

pub async fn resolve_member_id(
    id_or_mention: &str,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<uuid::Uuid, Box<dyn Error>> {
    if let Some(mention) = id_or_mention.strip_prefix('@') {
        if let Some(cache) = read_cache(cache_dir)
            && let Some(uuid_str) = cache.get(mention)
            && let Ok(uuid) = uuid_str.parse::<uuid::Uuid>()
        {
            return Ok(uuid);
        }

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

// --- Cache helpers ---

fn cache_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("member_cache.json")
}

pub(crate) fn read_cache(cache_dir: &Path) -> Option<HashMap<String, String>> {
    let path = cache_path(cache_dir);
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn write_cache(
    members: &progenitor_client::ResponseValue<Vec<api::types::Member>>,
    cache_dir: &Path,
) {
    let path = cache_path(cache_dir);

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

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
}
