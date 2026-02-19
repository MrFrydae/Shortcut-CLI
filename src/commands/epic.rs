use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};

use crate::api;

#[derive(Args)]
pub struct EpicArgs {
    #[command(subcommand)]
    pub action: EpicAction,
}

#[derive(Subcommand)]
pub enum EpicAction {
    /// List all epics
    List {
        /// Include epic descriptions in output
        #[arg(long, visible_alias = "descriptions")]
        desc: bool,
    },
    /// Update an epic
    Update(Box<UpdateArgs>),
}

#[derive(Args)]
pub struct UpdateArgs {
    /// The ID of the epic to update
    #[arg(long)]
    pub id: i64,

    /// The epic's name
    #[arg(long)]
    pub name: Option<String>,

    /// The epic's description
    #[arg(long)]
    pub description: Option<String>,

    /// The epic's deadline (RFC 3339 format, e.g. 2025-12-31T00:00:00Z)
    #[arg(long)]
    pub deadline: Option<String>,

    /// Whether the epic is archived
    #[arg(long)]
    pub archived: Option<bool>,

    /// The epic state ID or name (e.g. 500000042 or "in_progress")
    #[arg(long)]
    pub epic_state_id: Option<String>,

    /// Label names to attach (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,

    /// Objective IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub objective_ids: Vec<i64>,

    /// Owner member UUIDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub owner_ids: Vec<uuid::Uuid>,

    /// Follower member UUIDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub follower_ids: Vec<uuid::Uuid>,

    /// The UUID of the member that requested the epic
    #[arg(long)]
    pub requested_by_id: Option<uuid::Uuid>,
}

pub async fn run(
    args: &EpicArgs,
    client: &api::Client,
    cache_dir: PathBuf,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        EpicAction::List { desc } => run_list(*desc, client).await,
        EpicAction::Update(update_args) => run_update(update_args, client, &cache_dir).await,
    }
}

async fn run_list(desc: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let mut req = client.list_epics();
    if desc {
        req = req.includes_description(true);
    }
    let epics = req
        .send()
        .await
        .map_err(|e| format!("Failed to list epics: {e}"))?;
    for epic in epics.iter() {
        println!("{} - {}", epic.id, epic.name);
        if desc && let Some(d) = &epic.description {
            println!("  {}", d);
        }
    }
    Ok(())
}

async fn run_update(
    args: &UpdateArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateEpicName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateEpicDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let deadline = args
        .deadline
        .as_ref()
        .map(|d| chrono::DateTime::parse_from_rfc3339(d).map(|dt| dt.with_timezone(&chrono::Utc)))
        .transpose()
        .map_err(|e| format!("Invalid deadline: {e}"))?;

    let labels: Vec<api::types::CreateLabelParams> = args
        .labels
        .iter()
        .map(|n| -> Result<_, String> {
            Ok(api::types::CreateLabelParams {
                name: n.parse().map_err(|e| format!("Invalid label name: {e}"))?,
                color: None,
                description: None,
                external_id: None,
            })
        })
        .collect::<Result<_, _>>()?;

    let resolved_state_id = match &args.epic_state_id {
        Some(val) => Some(resolve_epic_state_id(val, client, cache_dir).await?),
        None => None,
    };

    let epic = client
        .update_epic()
        .epic_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(dl) = deadline {
                b = b.deadline(Some(dl));
            }
            if let Some(archived) = args.archived {
                b = b.archived(Some(archived));
            }
            if let Some(state_id) = resolved_state_id {
                b = b.epic_state_id(Some(state_id));
            }
            if !labels.is_empty() {
                b = b.labels(labels);
            }
            if !args.objective_ids.is_empty() {
                b = b.objective_ids(args.objective_ids.clone());
            }
            if !args.owner_ids.is_empty() {
                b = b.owner_ids(args.owner_ids.clone());
            }
            if !args.follower_ids.is_empty() {
                b = b.follower_ids(args.follower_ids.clone());
            }
            if let Some(req_id) = args.requested_by_id {
                b = b.requested_by_id(Some(req_id));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update epic: {e}"))?;

    println!("Updated epic {} - {}", epic.id, epic.name);
    Ok(())
}

// --- Name normalization ---

fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// --- Epic state resolution ---

async fn resolve_epic_state_id(
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

// --- Cache helpers ---

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
