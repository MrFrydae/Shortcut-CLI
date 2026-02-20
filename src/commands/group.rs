use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};

use crate::api;

use super::member;

#[derive(Args)]
pub struct GroupArgs {
    #[command(subcommand)]
    pub action: GroupAction,
}

#[derive(Subcommand)]
pub enum GroupAction {
    /// List all groups
    List {
        /// Include archived groups
        #[arg(long)]
        archived: bool,
    },
    /// Create a new group
    Create(Box<CreateArgs>),
    /// Get a group by ID
    Get {
        /// Group @mention_name or UUID
        #[arg(long)]
        id: String,
    },
    /// Update a group
    Update(Box<UpdateArgs>),
    /// List stories for a group
    Stories {
        /// Group @mention_name or UUID
        #[arg(long)]
        id: String,
        /// Maximum number of stories to return
        #[arg(long)]
        limit: Option<i64>,
        /// Offset for pagination
        #[arg(long)]
        offset: Option<i64>,
        /// Include story descriptions in output
        #[arg(long, visible_alias = "descriptions")]
        desc: bool,
    },
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    /// The group name
    #[arg(long)]
    pub name: String,

    /// The @mention name for the group
    #[arg(long)]
    pub mention_name: String,

    /// The group description
    #[arg(long)]
    pub description: Option<String>,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// Member(s) by @mention_name or UUID (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub member_ids: Vec<String>,

    /// Workflow IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub workflow_ids: Vec<i64>,
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// Group @mention_name or UUID
    #[arg(long)]
    pub id: String,

    /// The group name
    #[arg(long)]
    pub name: Option<String>,

    /// The @mention name for the group
    #[arg(long)]
    pub mention_name: Option<String>,

    /// The group description
    #[arg(long)]
    pub description: Option<String>,

    /// Whether the group is archived
    #[arg(long)]
    pub archived: Option<bool>,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// Member(s) by @mention_name or UUID (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub member_ids: Vec<String>,

    /// Workflow IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub workflow_ids: Vec<i64>,
}

pub async fn run(
    args: &GroupArgs,
    client: &api::Client,
    cache_dir: PathBuf,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        GroupAction::List { archived } => run_list(*archived, client, &cache_dir).await,
        GroupAction::Create(create_args) => run_create(create_args, client, &cache_dir).await,
        GroupAction::Get { id } => run_get(id, client, &cache_dir).await,
        GroupAction::Update(update_args) => run_update(update_args, client, &cache_dir).await,
        GroupAction::Stories {
            id,
            limit,
            offset,
            desc,
        } => run_stories(id, *limit, *offset, *desc, client, &cache_dir).await,
    }
}

async fn resolve_group_id(
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

async fn run_list(
    include_archived: bool,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let groups = client
        .list_groups()
        .send()
        .await
        .map_err(|e| format!("Failed to list groups: {e}"))?;

    for group in groups.iter() {
        if !include_archived && group.archived {
            continue;
        }
        println!(
            "{} - {} (@{}, {} members)",
            group.id,
            group.name,
            group.mention_name.as_str(),
            group.member_ids.len()
        );
    }

    write_cache(&groups, cache_dir);

    Ok(())
}

async fn run_create(
    args: &CreateArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .parse::<api::types::CreateGroupName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let mention_name = args
        .mention_name
        .parse::<api::types::CreateGroupMentionName>()
        .map_err(|e| format!("Invalid mention name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateGroupDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let member_ids = resolve_members(&args.member_ids, client, cache_dir).await?;

    let group = client
        .create_group()
        .body_map(|mut b| {
            b = b.name(name).mention_name(mention_name);
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if !member_ids.is_empty() {
                b = b.member_ids(member_ids);
            }
            if !args.workflow_ids.is_empty() {
                b = b.workflow_ids(args.workflow_ids.clone());
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create group: {e}"))?;

    println!(
        "Created group {} - {} (@{})",
        group.id,
        group.name,
        group.mention_name.as_str()
    );
    Ok(())
}

async fn run_get(id: &str, client: &api::Client, cache_dir: &Path) -> Result<(), Box<dyn Error>> {
    let group_id = resolve_group_id(id, client, cache_dir).await?;

    let group = client
        .get_group()
        .group_public_id(group_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get group: {e}"))?;

    println!(
        "{} - {} (@{})",
        group.id,
        group.name,
        group.mention_name.as_str()
    );
    println!("  Description: {}", group.description);
    println!("  Archived:    {}", group.archived);
    println!(
        "  Color:       {}",
        group.color.as_deref().unwrap_or("none")
    );
    println!("  Members:     {}", group.member_ids.len());

    if !group.member_ids.is_empty() {
        let members = client
            .list_members()
            .send()
            .await
            .map_err(|e| format!("Failed to list members: {e}"))?;

        let member_map: HashMap<uuid::Uuid, _> = members.iter().map(|m| (m.id, m)).collect();

        for member_id in &group.member_ids {
            if let Some(m) = member_map.get(member_id) {
                let name = m.profile.name.as_deref().unwrap_or("");
                println!("    @{} - {} ({})", m.profile.mention_name, name, m.role);
            } else {
                println!("    {member_id}");
            }
        }
    }

    println!(
        "  Stories:     {} total, {} started, {} backlog",
        group.num_stories, group.num_stories_started, group.num_stories_backlog
    );
    println!("  Epics:       {} started", group.num_epics_started);

    Ok(())
}

async fn run_update(
    args: &UpdateArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let group_id = resolve_group_id(&args.id, client, cache_dir).await?;

    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateGroupName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let mention_name = args
        .mention_name
        .as_ref()
        .map(|m| m.parse::<api::types::UpdateGroupMentionName>())
        .transpose()
        .map_err(|e| format!("Invalid mention name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateGroupDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let member_ids = resolve_members(&args.member_ids, client, cache_dir).await?;

    let group = client
        .update_group()
        .group_public_id(group_id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(mention) = mention_name {
                b = b.mention_name(Some(mention));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(archived) = args.archived {
                b = b.archived(Some(archived));
            }
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if !member_ids.is_empty() {
                b = b.member_ids(member_ids);
            }
            if !args.workflow_ids.is_empty() {
                b = b.workflow_ids(args.workflow_ids.clone());
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update group: {e}"))?;

    println!("Updated group {} - {}", group.id, group.name);
    Ok(())
}

async fn run_stories(
    id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
    desc: bool,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let group_id = resolve_group_id(id, client, cache_dir).await?;

    let mut req = client.list_group_stories().group_public_id(group_id);
    if let Some(limit) = limit {
        req = req.limit(limit);
    }
    if let Some(offset) = offset {
        req = req.offset(offset);
    }
    let stories = req
        .send()
        .await
        .map_err(|e| format!("Failed to list group stories: {e}"))?;

    for story in stories.iter() {
        println!(
            "{} - {} ({}, state_id: {})",
            story.id, story.name, story.story_type, story.workflow_state_id
        );
        if desc && let Some(d) = &story.description {
            println!("  {d}");
        }
    }

    if stories.is_empty() {
        println!("No stories in this group");
    }

    Ok(())
}

// --- Member resolution ---

async fn resolve_members(
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

// --- Group cache helpers ---

fn cache_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("group_cache.json")
}

fn read_cache(cache_dir: &Path) -> Option<HashMap<String, String>> {
    let path = cache_path(cache_dir);
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn write_cache(
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
