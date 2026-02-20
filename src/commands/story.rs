use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};

use crate::api;

use super::member;
use super::task;

#[derive(Args)]
pub struct StoryArgs {
    #[command(subcommand)]
    pub action: StoryAction,
}

#[derive(Subcommand)]
pub enum StoryAction {
    /// Create a new story
    Create(Box<CreateArgs>),
    /// Update an existing story
    Update(Box<UpdateArgs>),
    /// Get a story by ID
    Get {
        /// The ID of the story
        #[arg(long)]
        id: i64,
    },
    /// Manage checklist tasks on a story
    Task(task::TaskArgs),
}

#[derive(Args)]
pub struct CreateArgs {
    /// The name of the story
    #[arg(long)]
    pub name: String,

    /// The description of the story
    #[arg(long)]
    pub description: Option<String>,

    /// The type of story (feature, bug, chore)
    #[arg(long, name = "type")]
    pub story_type: Option<String>,

    /// Owner(s) by @mention_name or UUID (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub owner: Vec<String>,

    /// The workflow state name or ID
    #[arg(long)]
    pub state: Option<String>,

    /// The epic ID to associate with
    #[arg(long)]
    pub epic_id: Option<i64>,

    /// The story point estimate
    #[arg(long)]
    pub estimate: Option<i64>,

    /// Label names (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// The ID of the story to update
    #[arg(long)]
    pub id: i64,

    /// The name of the story
    #[arg(long)]
    pub name: Option<String>,

    /// The description of the story
    #[arg(long)]
    pub description: Option<String>,

    /// The type of story (feature, bug, chore)
    #[arg(long, name = "type")]
    pub story_type: Option<String>,

    /// Owner(s) by @mention_name or UUID (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub owner: Vec<String>,

    /// The workflow state name or ID
    #[arg(long)]
    pub state: Option<String>,

    /// The epic ID to associate with
    #[arg(long)]
    pub epic_id: Option<i64>,

    /// The story point estimate
    #[arg(long)]
    pub estimate: Option<i64>,

    /// Label names (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,
}

pub async fn run(
    args: &StoryArgs,
    client: &api::Client,
    cache_dir: PathBuf,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        StoryAction::Create(create_args) => run_create(create_args, client, &cache_dir).await,
        StoryAction::Update(update_args) => run_update(update_args, client, &cache_dir).await,
        StoryAction::Get { id } => run_get(*id, client).await,
        StoryAction::Task(task_args) => task::run(task_args, client).await,
    }
}

async fn run_create(
    args: &CreateArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .parse::<api::types::CreateStoryParamsName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateStoryParamsDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let story_type = args
        .story_type
        .as_ref()
        .map(|t| t.parse::<api::types::CreateStoryParamsStoryType>())
        .transpose()
        .map_err(|e| format!("Invalid story type: {e}"))?;

    let owner_ids = resolve_owners(&args.owner, client, cache_dir).await?;

    let resolved_state_id = match &args.state {
        Some(val) => Some(resolve_workflow_state_id(val, client, cache_dir).await?),
        None => None,
    };

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

    let story = client
        .create_story()
        .body_map(|mut b| {
            b = b.name(name);
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(st) = story_type {
                b = b.story_type(Some(st));
            }
            if !owner_ids.is_empty() {
                b = b.owner_ids(Some(owner_ids));
            }
            if let Some(state_id) = resolved_state_id {
                b = b.workflow_state_id(Some(state_id));
            }
            if let Some(epic_id) = args.epic_id {
                b = b.epic_id(Some(epic_id));
            }
            if let Some(estimate) = args.estimate {
                b = b.estimate(Some(estimate));
            }
            if !labels.is_empty() {
                b = b.labels(labels);
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create story: {e}"))?;

    println!("Created story {} - {}", story.id, story.name);
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
        .map(|n| n.parse::<api::types::UpdateStoryName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateStoryDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let story_type = args
        .story_type
        .as_ref()
        .map(|t| t.parse::<api::types::UpdateStoryStoryType>())
        .transpose()
        .map_err(|e| format!("Invalid story type: {e}"))?;

    let owner_ids = resolve_owners(&args.owner, client, cache_dir).await?;

    let resolved_state_id = match &args.state {
        Some(val) => Some(resolve_workflow_state_id(val, client, cache_dir).await?),
        None => None,
    };

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

    let story = client
        .update_story()
        .story_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(st) = story_type {
                b = b.story_type(Some(st));
            }
            if !owner_ids.is_empty() {
                b = b.owner_ids(Some(owner_ids));
            }
            if let Some(state_id) = resolved_state_id {
                b = b.workflow_state_id(Some(state_id));
            }
            if let Some(epic_id) = args.epic_id {
                b = b.epic_id(Some(epic_id));
            }
            if let Some(estimate) = args.estimate {
                b = b.estimate(Some(estimate));
            }
            if !labels.is_empty() {
                b = b.labels(labels);
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update story: {e}"))?;

    println!("Updated story {} - {}", story.id, story.name);
    Ok(())
}

async fn run_get(id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let story = client
        .get_story()
        .story_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get story: {e}"))?;

    println!("{} - {}", story.id, story.name);
    println!("  Type:        {}", story.story_type);
    println!("  State ID:    {}", story.workflow_state_id);
    println!("  Workflow ID: {}", story.workflow_id);
    if let Some(epic_id) = story.epic_id {
        println!("  Epic ID:     {epic_id}");
    }
    if let Some(estimate) = story.estimate {
        println!("  Estimate:    {estimate}");
    }
    if !story.owner_ids.is_empty() {
        let ids: Vec<String> = story.owner_ids.iter().map(|id| id.to_string()).collect();
        println!("  Owners:      {}", ids.join(", "));
    }
    if !story.labels.is_empty() {
        let names: Vec<&str> = story.labels.iter().map(|l| l.name.as_str()).collect();
        println!("  Labels:      {}", names.join(", "));
    }
    if !story.description.is_empty() {
        println!("  Description: {}", story.description);
    }

    Ok(())
}

// --- Owner resolution ---

async fn resolve_owners(
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

fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// --- Workflow state resolution ---

async fn resolve_workflow_state_id(
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
    let workflows = client
        .list_workflows()
        .send()
        .await
        .map_err(|e| format!("Failed to list workflows: {e}"))?;

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
