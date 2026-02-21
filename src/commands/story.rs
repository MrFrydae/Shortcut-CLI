use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};

use crate::api;

use super::custom_field;
use super::member;
use super::story_comment;
use super::story_link;
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
    /// List/search stories
    List(Box<ListArgs>),
    /// Delete a story
    Delete {
        /// The ID of the story to delete
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// Manage checklist tasks on a story
    Task(task::TaskArgs),
    /// Manage story links (relationships between stories)
    Link(story_link::LinkArgs),
    /// Manage comments on a story
    Comment(story_comment::CommentArgs),
    /// Show the change history of a story
    History(HistoryArgs),
}

#[derive(Args)]
pub struct HistoryArgs {
    /// The ID of the story
    #[arg(long)]
    pub id: i64,
    /// Maximum number of history entries to display
    #[arg(long)]
    pub limit: Option<usize>,
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
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

    /// The iteration ID to assign this story to
    #[arg(long)]
    pub iteration_id: Option<i64>,

    /// Set a custom field value (format: "FieldName=Value", repeatable)
    #[arg(long = "custom-field")]
    pub custom_fields: Vec<String>,
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
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

    /// The iteration ID to assign this story to
    #[arg(long)]
    pub iteration_id: Option<i64>,

    /// Set a custom field value (format: "FieldName=Value", repeatable)
    #[arg(long = "custom-field")]
    pub custom_fields: Vec<String>,
}

#[derive(Args)]
pub struct ListArgs {
    /// Filter by owner (@mention_name or UUID)
    #[arg(long)]
    pub owner: Option<String>,

    /// Filter by workflow state name or ID
    #[arg(long)]
    pub state: Option<String>,

    /// Filter by epic ID
    #[arg(long)]
    pub epic_id: Option<i64>,

    /// Filter by story type (feature, bug, chore)
    #[arg(long, name = "type")]
    pub story_type: Option<String>,

    /// Filter by label name
    #[arg(long)]
    pub label: Option<String>,

    /// Filter by project ID
    #[arg(long)]
    pub project_id: Option<i64>,

    /// Maximum number of stories to display (default 25)
    #[arg(long, default_value = "25")]
    pub limit: i64,

    /// Include story descriptions in output
    #[arg(long, visible_alias = "descriptions")]
    pub desc: bool,
}

pub async fn run(
    args: &StoryArgs,
    client: &api::Client,
    cache_dir: PathBuf,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        StoryAction::Create(create_args) => run_create(create_args, client, &cache_dir).await,
        StoryAction::Update(update_args) => run_update(update_args, client, &cache_dir).await,
        StoryAction::Get { id } => run_get(*id, client, &cache_dir).await,
        StoryAction::List(list_args) => run_list(list_args, client, &cache_dir).await,
        StoryAction::Delete { id, confirm } => run_delete(*id, *confirm, client).await,
        StoryAction::Task(task_args) => task::run(task_args, client).await,
        StoryAction::Link(link_args) => story_link::run(link_args, client).await,
        StoryAction::Comment(args) => story_comment::run(args, client, &cache_dir).await,
        StoryAction::History(history_args) => run_history(history_args, client, &cache_dir).await,
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

    let custom_field_params =
        resolve_custom_field_args(&args.custom_fields, client, cache_dir).await?;

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
            if let Some(iter_id) = args.iteration_id {
                b = b.iteration_id(Some(iter_id));
            }
            if !custom_field_params.is_empty() {
                b = b.custom_fields(custom_field_params);
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

    let custom_field_params =
        resolve_custom_field_args(&args.custom_fields, client, cache_dir).await?;

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
            if let Some(iter_id) = args.iteration_id {
                b = b.iteration_id(Some(iter_id));
            }
            if !custom_field_params.is_empty() {
                b = b.custom_fields(custom_field_params);
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update story: {e}"))?;

    println!("Updated story {} - {}", story.id, story.name);
    Ok(())
}

async fn run_get(id: i64, client: &api::Client, cache_dir: &Path) -> Result<(), Box<dyn Error>> {
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
    if !story.story_links.is_empty() {
        println!("  Links:");
        for link in &story.story_links {
            let (display_verb, other_id) = if link.type_ == "subject" {
                (link.verb.as_str(), link.object_id)
            } else {
                (story_link::invert_verb(&link.verb), link.subject_id)
            };
            println!("    {display_verb} {other_id} (link {})", link.id);
        }
    }
    if !story.custom_fields.is_empty() {
        let field_ids: Vec<uuid::Uuid> = story.custom_fields.iter().map(|cf| cf.field_id).collect();
        let names = custom_field::resolve_custom_field_names(&field_ids, client, cache_dir).await?;
        for cf in &story.custom_fields {
            let field_name = names
                .get(&cf.field_id.to_string())
                .map(|s| s.as_str())
                .unwrap_or("Unknown");
            println!("  {}: {}", field_name, cf.value);
        }
    }

    Ok(())
}

async fn run_list(
    args: &ListArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let resolved_owner_id = match &args.owner {
        Some(val) => Some(member::resolve_member_id(val, client, cache_dir).await?),
        None => None,
    };

    let resolved_state_id = match &args.state {
        Some(val) => Some(resolve_workflow_state_id(val, client, cache_dir).await?),
        None => None,
    };

    let resolved_story_type = args
        .story_type
        .as_ref()
        .map(|t| t.parse::<api::types::SearchStoriesStoryType>())
        .transpose()
        .map_err(|e| format!("Invalid story type: {e}"))?;

    let resolved_label_name = args
        .label
        .as_ref()
        .map(|l| l.parse::<api::types::SearchStoriesLabelName>())
        .transpose()
        .map_err(|e| format!("Invalid label name: {e}"))?;

    let include_desc = args.desc;
    let epic_id = args.epic_id;
    let project_id = args.project_id;

    let stories = client
        .query_stories()
        .body_map(|mut b| {
            if let Some(owner_id) = resolved_owner_id {
                b = b.owner_id(Some(owner_id));
            }
            if let Some(state_id) = resolved_state_id {
                b = b.workflow_state_id(Some(state_id));
            }
            if let Some(epic_id) = epic_id {
                b = b.epic_id(Some(epic_id));
            }
            if let Some(st) = resolved_story_type {
                b = b.story_type(Some(st));
            }
            if let Some(label) = resolved_label_name {
                b = b.label_name(Some(label));
            }
            if let Some(pid) = project_id {
                b = b.project_id(Some(pid));
            }
            if include_desc {
                b = b.includes_description(Some(true));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to search stories: {e}"))?;

    let limit = args.limit as usize;
    let mut count = 0;

    for story in stories.iter() {
        if count >= limit {
            break;
        }
        println!(
            "{} - {} ({}, state_id: {})",
            story.id, story.name, story.story_type, story.workflow_state_id
        );
        if args.desc
            && let Some(d) = &story.description
        {
            println!("  {d}");
        }
        count += 1;
    }

    if count == 0 {
        println!("No stories found");
    }

    Ok(())
}

async fn run_delete(id: i64, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a story is irreversible. Pass --confirm to proceed.".into());
    }

    let story = client
        .get_story()
        .story_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get story: {e}"))?;

    let name = story.name.clone();

    client
        .delete_story()
        .story_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete story: {e}"))?;

    println!("Deleted story {id} - {name}");
    Ok(())
}

// --- History ---

async fn run_history(
    args: &HistoryArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let entries = client
        .story_history()
        .story_public_id(args.id)
        .send()
        .await
        .map_err(|e| format!("Failed to get story history: {e}"))?;

    let mut entries: Vec<&api::types::History> = entries.iter().collect();
    entries.sort_by(|a, b| a.changed_at.cmp(&b.changed_at));

    if let Some(limit) = args.limit {
        let len = entries.len();
        if len > limit {
            entries = entries.split_off(len - limit);
        }
    }

    if entries.is_empty() {
        println!("No history found for story {}", args.id);
        return Ok(());
    }

    // Build member lookup: uuid -> @mention_name
    let member_map = build_member_lookup(cache_dir, client).await?;

    // Build state and label lookups from references
    let mut state_map: HashMap<i64, String> = HashMap::new();
    let mut label_map: HashMap<i64, String> = HashMap::new();
    for entry in &entries {
        for reference in &entry.references {
            match reference {
                api::types::HistoryReferencesItem::WorkflowState(ws) => {
                    if let api::types::HistoryReferenceWorkflowStateId::Variant0(id) = ws.id {
                        state_map.insert(id, ws.name.clone());
                    }
                }
                api::types::HistoryReferencesItem::Label(lb) => {
                    if let api::types::HistoryReferenceLabelId::Variant0(id) = lb.id {
                        label_map.insert(id, lb.name.clone());
                    }
                }
                _ => {}
            }
        }
    }

    for entry in &entries {
        let who = entry
            .member_id
            .as_ref()
            .and_then(|id| member_map.get(&id.to_string()))
            .map(|m| format!("@{m}"))
            .unwrap_or_else(|| "(system)".to_string());

        for action in &entry.actions {
            if let Some(line) = format_action(action, &state_map, &label_map, &member_map) {
                println!("[{}] {who}: {line}", entry.changed_at);
            }
        }
    }

    Ok(())
}

async fn build_member_lookup(
    cache_dir: &Path,
    client: &api::Client,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
    // Try cache first: mention_name -> uuid, we need uuid -> mention_name
    if let Some(cache) = member::read_cache(cache_dir) {
        let reverse: HashMap<String, String> = cache
            .into_iter()
            .map(|(mention, uuid)| (uuid, mention))
            .collect();
        if !reverse.is_empty() {
            return Ok(reverse);
        }
    }

    // Cache miss — fetch members
    let members = client
        .list_members()
        .send()
        .await
        .map_err(|e| format!("Failed to list members: {e}"))?;

    let map: HashMap<String, String> = members
        .iter()
        .map(|m| (m.id.to_string(), m.profile.mention_name.clone()))
        .collect();

    Ok(map)
}

fn format_action(
    action: &api::types::HistoryActionsItem,
    state_map: &HashMap<i64, String>,
    label_map: &HashMap<i64, String>,
    member_map: &HashMap<String, String>,
) -> Option<String> {
    use api::types::HistoryActionsItem::*;
    match action {
        StoryCreate(a) => Some(format!(
            "created story {} \"{}\" ({})",
            a.id, a.name, a.story_type
        )),
        StoryUpdate(a) => {
            let changes = format_story_update(a.changes.as_ref(), state_map, label_map, member_map);
            if changes.is_empty() {
                Some(format!("updated story {} \"{}\"", a.id, a.name))
            } else {
                Some(format!(
                    "updated story {} \"{}\": {}",
                    a.id, a.name, changes
                ))
            }
        }
        StoryDelete(a) => Some(format!(
            "deleted story {} \"{}\" ({})",
            a.id, a.name, a.story_type
        )),
        TaskCreate(a) => Some(format!("added task {} \"{}\"", a.id, a.description)),
        TaskUpdate(a) => Some(format!("updated task {} \"{}\"", a.id, a.description)),
        TaskDelete(a) => Some(format!("deleted task {} \"{}\"", a.id, a.description)),
        StoryCommentCreate(a) => Some(format!("added comment {}", a.id)),
        StoryLinkCreate(a) => Some(format!(
            "created link {} ({} {} -> {})",
            a.id, a.verb, a.subject_id, a.object_id
        )),
        StoryLinkUpdate(a) => Some(format!(
            "updated link {} ({} {} -> {})",
            a.id, a.verb, a.subject_id, a.object_id
        )),
        StoryLinkDelete(a) => Some(format!("deleted link {}", a.id)),
        LabelCreate(a) => Some(format!("added label \"{}\"", a.name)),
        LabelUpdate(a) => Some(format!("updated label {}", a.id)),
        LabelDelete(a) => Some(format!("removed label \"{}\"", a.name)),
        BranchCreate(a) => Some(format!("created branch \"{}\"", a.name)),
        BranchMerge(a) => Some(format!("merged branch \"{}\"", a.name)),
        BranchPush(a) => Some(format!("pushed to branch \"{}\"", a.name)),
        PullRequest(a) => Some(format!("pull request #{} \"{}\"", a.number, a.title)),
        ProjectUpdate(a) => Some(format!("updated project \"{}\"", a.name)),
        Workspace2BulkUpdate(a) => Some(format!("bulk update \"{}\"", a.name)),
    }
}

fn format_story_update(
    changes: Option<&api::types::HistoryChangesStory>,
    state_map: &HashMap<i64, String>,
    label_map: &HashMap<i64, String>,
    member_map: &HashMap<String, String>,
) -> String {
    let Some(c) = changes else {
        return String::new();
    };

    let mut parts: Vec<String> = Vec::new();

    if let Some(ws) = &c.workflow_state_id {
        let old_name = ws
            .old
            .and_then(|id| state_map.get(&id))
            .map(|s| s.as_str())
            .unwrap_or("(none)");
        let new_name = ws
            .new
            .and_then(|id| state_map.get(&id))
            .map(|s| s.as_str())
            .unwrap_or("(none)");
        parts.push(format!("state: {old_name} -> {new_name}"));
    }

    if let Some(n) = &c.name {
        let old = n.old.as_deref().unwrap_or("(none)");
        let new = n.new.as_deref().unwrap_or("(none)");
        parts.push(format!("name: \"{old}\" -> \"{new}\""));
    }

    if let Some(st) = &c.story_type {
        let old = st.old.as_deref().unwrap_or("(none)");
        let new = st.new.as_deref().unwrap_or("(none)");
        parts.push(format!("type: {old} -> {new}"));
    }

    if let Some(e) = &c.estimate {
        let old = e
            .old
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        let new = e
            .new
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        parts.push(format!("estimate: {old} -> {new}"));
    }

    if let Some(e) = &c.epic_id {
        let old = e
            .old
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        let new = e
            .new
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        parts.push(format!("epic_id: {old} -> {new}"));
    }

    if let Some(i) = &c.iteration_id {
        let old = i
            .old
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        let new = i
            .new
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        parts.push(format!("iteration_id: {old} -> {new}"));
    }

    if let Some(d) = &c.deadline {
        let old = d.old.as_deref().unwrap_or("(none)");
        let new = d.new.as_deref().unwrap_or("(none)");
        parts.push(format!("deadline: {old} -> {new}"));
    }

    if c.description.is_some() {
        parts.push("description: (changed)".to_string());
    }

    if let Some(l) = &c.label_ids {
        if !l.adds.is_empty() {
            let names: Vec<&str> = l
                .adds
                .iter()
                .map(|id| label_map.get(id).map(|s| s.as_str()).unwrap_or("unknown"))
                .collect();
            parts.push(format!("labels added: {}", names.join(", ")));
        }
        if !l.removes.is_empty() {
            let names: Vec<&str> = l
                .removes
                .iter()
                .map(|id| label_map.get(id).map(|s| s.as_str()).unwrap_or("unknown"))
                .collect();
            parts.push(format!("labels removed: {}", names.join(", ")));
        }
    }

    if let Some(o) = &c.owner_ids {
        if !o.adds.is_empty() {
            let names: Vec<String> = o
                .adds
                .iter()
                .map(|id| {
                    member_map
                        .get(&id.to_string())
                        .map(|m| format!("@{m}"))
                        .unwrap_or_else(|| id.to_string())
                })
                .collect();
            parts.push(format!("owners added: {}", names.join(", ")));
        }
        if !o.removes.is_empty() {
            let names: Vec<String> = o
                .removes
                .iter()
                .map(|id| {
                    member_map
                        .get(&id.to_string())
                        .map(|m| format!("@{m}"))
                        .unwrap_or_else(|| id.to_string())
                })
                .collect();
            parts.push(format!("owners removed: {}", names.join(", ")));
        }
    }

    if let Some(s) = &c.started {
        parts.push(format!(
            "started: {} -> {}",
            s.old.unwrap_or(false),
            s.new.unwrap_or(false)
        ));
    }

    if let Some(s) = &c.completed {
        parts.push(format!(
            "completed: {} -> {}",
            s.old.unwrap_or(false),
            s.new.unwrap_or(false)
        ));
    }

    if let Some(s) = &c.archived {
        parts.push(format!(
            "archived: {} -> {}",
            s.old.unwrap_or(false),
            s.new.unwrap_or(false)
        ));
    }

    if let Some(s) = &c.blocked {
        parts.push(format!(
            "blocked: {} -> {}",
            s.old.unwrap_or(false),
            s.new.unwrap_or(false)
        ));
    }

    if let Some(s) = &c.blocker {
        parts.push(format!(
            "blocker: {} -> {}",
            s.old.unwrap_or(false),
            s.new.unwrap_or(false)
        ));
    }

    if let Some(p) = &c.project_id {
        let old = p
            .old
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        let new = p
            .new
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        parts.push(format!("project_id: {old} -> {new}"));
    }

    if let Some(g) = &c.group_id {
        let old = g
            .old
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        let new = g
            .new
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        parts.push(format!("group_id: {old} -> {new}"));
    }

    parts.join(", ")
}

// --- Custom field argument parsing ---

fn parse_custom_field_arg(arg: &str) -> Result<(&str, &str), Box<dyn Error>> {
    let (name, value) = arg.split_once('=').ok_or_else(|| {
        format!("Invalid custom field format '{arg}': expected 'FieldName=Value'")
    })?;
    let name = name.trim();
    let value = value.trim();
    if name.is_empty() || value.is_empty() {
        return Err(format!(
            "Invalid custom field format '{arg}': name and value must not be empty"
        )
        .into());
    }
    Ok((name, value))
}

async fn resolve_custom_field_args(
    args: &[String],
    client: &api::Client,
    cache_dir: &Path,
) -> Result<Vec<api::types::CustomFieldValueParams>, Box<dyn Error>> {
    let mut params = Vec::with_capacity(args.len());
    for arg in args {
        let (field_name, value_name) = parse_custom_field_arg(arg)?;
        let (field_id, value_id) =
            custom_field::resolve_custom_field_value(field_name, value_name, client, cache_dir)
                .await?;
        params.push(api::types::CustomFieldValueParams {
            field_id,
            value: None,
            value_id,
        });
    }
    Ok(params)
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
