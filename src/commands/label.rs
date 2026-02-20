use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};

use crate::api;

#[derive(Args)]
pub struct LabelArgs {
    #[command(subcommand)]
    pub action: LabelAction,
}

#[derive(Subcommand)]
pub enum LabelAction {
    /// List all labels
    List {
        /// Include label descriptions in output
        #[arg(long, visible_alias = "descriptions")]
        desc: bool,
    },
    /// Create a new label
    Create(Box<CreateArgs>),
    /// Get a label by ID
    Get {
        /// The ID of the label
        #[arg(long)]
        id: i64,
    },
    /// Update a label
    Update(Box<UpdateArgs>),
    /// Delete a label
    Delete {
        /// The ID of the label to delete
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// List stories with a label
    Stories {
        /// The ID of the label
        #[arg(long)]
        id: i64,
        /// Include story descriptions in output
        #[arg(long, visible_alias = "descriptions")]
        desc: bool,
    },
    /// List epics with a label
    Epics {
        /// The ID of the label
        #[arg(long)]
        id: i64,
        /// Include epic descriptions in output
        #[arg(long, visible_alias = "descriptions")]
        desc: bool,
    },
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    /// The label name
    #[arg(long)]
    pub name: String,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// The label description
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// The ID of the label to update
    #[arg(long)]
    pub id: i64,

    /// The label name
    #[arg(long)]
    pub name: Option<String>,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// The label description
    #[arg(long)]
    pub description: Option<String>,

    /// Whether the label is archived
    #[arg(long)]
    pub archived: Option<bool>,
}

pub async fn run(
    args: &LabelArgs,
    client: &api::Client,
    cache_dir: PathBuf,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        LabelAction::List { desc } => run_list(*desc, client, &cache_dir).await,
        LabelAction::Create(create_args) => run_create(create_args, client, &cache_dir).await,
        LabelAction::Get { id } => run_get(*id, client).await,
        LabelAction::Update(update_args) => run_update(update_args, client).await,
        LabelAction::Delete { id, confirm } => run_delete(*id, *confirm, client).await,
        LabelAction::Stories { id, desc } => run_stories(*id, *desc, client).await,
        LabelAction::Epics { id, desc } => run_epics(*id, *desc, client).await,
    }
}

async fn run_list(
    desc: bool,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let labels = client
        .list_labels()
        .slim(false)
        .send()
        .await
        .map_err(|e| format!("Failed to list labels: {e}"))?;

    for label in labels.iter() {
        let color = label
            .color
            .as_deref()
            .map(|c| format!(" ({c})"))
            .unwrap_or_default();
        println!("{} - {}{}", label.id, label.name, color);
        if desc && let Some(d) = &label.description {
            println!("  {d}");
        }
    }

    // Update label cache
    let map: HashMap<String, i64> = labels
        .iter()
        .map(|l| (normalize_name(&l.name), l.id))
        .collect();
    write_cache(&map, cache_dir);

    Ok(())
}

async fn run_create(
    args: &CreateArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .parse::<api::types::CreateLabelParamsName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateLabelParamsDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let label = client
        .create_label()
        .body_map(|mut b| {
            b = b.name(name);
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create label: {e}"))?;

    let color = label
        .color
        .as_deref()
        .map(|c| format!(" ({c})"))
        .unwrap_or_default();
    println!("Created label {} - {}{}", label.id, label.name, color);

    // Update cache with new label
    let mut map = read_cache(cache_dir).unwrap_or_default();
    map.insert(normalize_name(&label.name), label.id);
    write_cache(&map, cache_dir);

    Ok(())
}

async fn run_get(id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let label = client
        .get_label()
        .label_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get label: {e}"))?;

    let color = label
        .color
        .as_deref()
        .map(|c| format!(" ({c})"))
        .unwrap_or_default();
    println!("{} - {}{}", label.id, label.name, color);
    if let Some(d) = &label.description {
        println!("  Description: {d}");
    }
    println!("  Archived:    {}", label.archived);
    if let Some(stats) = &label.stats {
        println!("  Stories:     {}", stats.num_stories_total);
        println!("  Epics:       {}", stats.num_epics_total);
    }

    Ok(())
}

async fn run_update(args: &UpdateArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateLabelName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateLabelDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let label = client
        .update_label()
        .label_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(archived) = args.archived {
                b = b.archived(Some(archived));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update label: {e}"))?;

    println!("Updated label {} - {}", label.id, label.name);
    Ok(())
}

async fn run_delete(id: i64, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a label is irreversible. Pass --confirm to proceed.".into());
    }

    let label = client
        .get_label()
        .label_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get label: {e}"))?;

    let name = label.name.clone();

    client
        .delete_label()
        .label_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete label: {e}"))?;

    println!("Deleted label {id} - {name}");
    Ok(())
}

async fn run_stories(id: i64, desc: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let mut req = client.list_label_stories().label_public_id(id);
    if desc {
        req = req.includes_description(true);
    }
    let stories = req
        .send()
        .await
        .map_err(|e| format!("Failed to list label stories: {e}"))?;

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
        println!("No stories with this label");
    }

    Ok(())
}

async fn run_epics(id: i64, desc: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let epics = client
        .list_label_epics()
        .label_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list label epics: {e}"))?;

    for epic in epics.iter() {
        println!("{} - {}", epic.id, epic.name);
        if desc && let Some(d) = &epic.description {
            println!("  {d}");
        }
    }

    if epics.is_empty() {
        println!("No epics with this label");
    }

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

// --- Label cache ---

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

// --- Cache helpers ---

fn cache_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("label_cache.json")
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
