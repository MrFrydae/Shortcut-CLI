use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};

use crate::api;

use super::member;

#[derive(Args)]
pub struct IterationArgs {
    #[command(subcommand)]
    pub action: IterationAction,
}

#[derive(Subcommand)]
pub enum IterationAction {
    /// List all iterations
    List {
        /// Filter by status (e.g. "started", "unstarted", "done")
        #[arg(long)]
        state: Option<String>,
    },
    /// Create a new iteration
    Create(Box<CreateArgs>),
    /// Get an iteration by ID
    Get {
        /// The ID of the iteration
        #[arg(long)]
        id: i64,
    },
    /// Update an iteration
    Update(Box<UpdateArgs>),
    /// Delete an iteration
    Delete {
        /// The ID of the iteration to delete
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// List stories in an iteration
    Stories {
        /// The ID of the iteration
        #[arg(long)]
        id: i64,
        /// Include story descriptions in output
        #[arg(long, visible_alias = "descriptions")]
        desc: bool,
    },
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    /// The iteration name
    #[arg(long)]
    pub name: String,

    /// The start date (YYYY-MM-DD)
    #[arg(long)]
    pub start_date: String,

    /// The end date (YYYY-MM-DD)
    #[arg(long)]
    pub end_date: String,

    /// The iteration description
    #[arg(long)]
    pub description: Option<String>,

    /// Follower(s) by @mention_name or UUID (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub followers: Vec<String>,

    /// Label names (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,

    /// Group IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub group_ids: Vec<uuid::Uuid>,
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// The ID of the iteration to update
    #[arg(long)]
    pub id: i64,

    /// The iteration name
    #[arg(long)]
    pub name: Option<String>,

    /// The start date (YYYY-MM-DD)
    #[arg(long)]
    pub start_date: Option<String>,

    /// The end date (YYYY-MM-DD)
    #[arg(long)]
    pub end_date: Option<String>,

    /// The iteration description
    #[arg(long)]
    pub description: Option<String>,

    /// Follower(s) by @mention_name or UUID (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub followers: Vec<String>,

    /// Label names (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,

    /// Group IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub group_ids: Vec<uuid::Uuid>,
}

pub async fn run(
    args: &IterationArgs,
    client: &api::Client,
    cache_dir: PathBuf,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        IterationAction::List { state } => run_list(state.as_deref(), client).await,
        IterationAction::Create(create_args) => run_create(create_args, client, &cache_dir).await,
        IterationAction::Get { id } => run_get(*id, client).await,
        IterationAction::Update(update_args) => run_update(update_args, client, &cache_dir).await,
        IterationAction::Delete { id, confirm } => run_delete(*id, *confirm, client).await,
        IterationAction::Stories { id, desc } => run_stories(*id, *desc, client).await,
    }
}

async fn run_list(state: Option<&str>, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let iterations = client
        .list_iterations()
        .send()
        .await
        .map_err(|e| format!("Failed to list iterations: {e}"))?;

    for iter in iterations.iter() {
        let status = iter.status.to_string();
        if let Some(filter) = state
            && !status.eq_ignore_ascii_case(filter)
        {
            continue;
        }
        println!(
            "{} - {} ({}, {} \u{2192} {})",
            iter.id, iter.name, status, iter.start_date, iter.end_date
        );
    }
    Ok(())
}

async fn run_create(
    args: &CreateArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    chrono::NaiveDate::parse_from_str(&args.start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start date format (expected YYYY-MM-DD): {e}"))?;
    chrono::NaiveDate::parse_from_str(&args.end_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end date format (expected YYYY-MM-DD): {e}"))?;

    let name = args
        .name
        .parse::<api::types::CreateIterationName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateIterationDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let start_date = args
        .start_date
        .parse::<api::types::CreateIterationStartDate>()
        .map_err(|e| format!("Invalid start date: {e}"))?;

    let end_date = args
        .end_date
        .parse::<api::types::CreateIterationEndDate>()
        .map_err(|e| format!("Invalid end date: {e}"))?;

    let follower_ids = resolve_followers(&args.followers, client, cache_dir).await?;

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

    let iteration = client
        .create_iteration()
        .body_map(|mut b| {
            b = b.name(name).start_date(start_date).end_date(end_date);
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if !follower_ids.is_empty() {
                b = b.follower_ids(follower_ids);
            }
            if !labels.is_empty() {
                b = b.labels(labels);
            }
            if !args.group_ids.is_empty() {
                b = b.group_ids(args.group_ids.clone());
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create iteration: {e}"))?;

    println!(
        "Created iteration {} - {} ({} \u{2192} {})",
        iteration.id, iteration.name, iteration.start_date, iteration.end_date
    );
    Ok(())
}

async fn run_get(id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let iteration = client
        .get_iteration()
        .iteration_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get iteration: {e}"))?;

    println!("{} - {}", iteration.id, iteration.name);
    println!("  Status:      {}", iteration.status);
    println!("  Start:       {}", iteration.start_date);
    println!("  End:         {}", iteration.end_date);
    if !iteration.labels.is_empty() {
        let names: Vec<&str> = iteration.labels.iter().map(|l| l.name.as_str()).collect();
        println!("  Labels:      {}", names.join(", "));
    }

    let stats = &iteration.stats;
    let total_stories = stats.num_stories_started
        + stats.num_stories_done
        + stats.num_stories_unstarted
        + stats.num_stories_backlog;
    println!(
        "  Stories:     {} total, {} started, {} done, {} unstarted",
        total_stories,
        stats.num_stories_started,
        stats.num_stories_done,
        stats.num_stories_unstarted,
    );
    println!(
        "  Points:      {} total, {} started, {} done, {} unstarted",
        stats.num_points,
        stats.num_points_started,
        stats.num_points_done,
        stats.num_points_unstarted,
    );

    if let Some(cycle) = stats.average_cycle_time {
        let cycle_days = cycle as f64 / 86400.0;
        println!("  Avg Cycle:   {cycle_days:.1} days");
    }
    if let Some(lead) = stats.average_lead_time {
        let lead_days = lead as f64 / 86400.0;
        println!("  Avg Lead:    {lead_days:.1} days");
    }

    if !iteration.description.is_empty() {
        println!("  Description: {}", iteration.description);
    }

    Ok(())
}

async fn run_update(
    args: &UpdateArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    if let Some(ref sd) = args.start_date {
        chrono::NaiveDate::parse_from_str(sd, "%Y-%m-%d")
            .map_err(|e| format!("Invalid start date format (expected YYYY-MM-DD): {e}"))?;
    }
    if let Some(ref ed) = args.end_date {
        chrono::NaiveDate::parse_from_str(ed, "%Y-%m-%d")
            .map_err(|e| format!("Invalid end date format (expected YYYY-MM-DD): {e}"))?;
    }

    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateIterationName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateIterationDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let start_date = args
        .start_date
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateIterationStartDate>())
        .transpose()
        .map_err(|e| format!("Invalid start date: {e}"))?;

    let end_date = args
        .end_date
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateIterationEndDate>())
        .transpose()
        .map_err(|e| format!("Invalid end date: {e}"))?;

    let follower_ids = resolve_followers(&args.followers, client, cache_dir).await?;

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

    let iteration = client
        .update_iteration()
        .iteration_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(sd) = start_date {
                b = b.start_date(Some(sd));
            }
            if let Some(ed) = end_date {
                b = b.end_date(Some(ed));
            }
            if !follower_ids.is_empty() {
                b = b.follower_ids(follower_ids);
            }
            if !labels.is_empty() {
                b = b.labels(labels);
            }
            if !args.group_ids.is_empty() {
                b = b.group_ids(args.group_ids.clone());
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update iteration: {e}"))?;

    println!("Updated iteration {} - {}", iteration.id, iteration.name);
    Ok(())
}

async fn run_delete(id: i64, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting an iteration is irreversible. Pass --confirm to proceed.".into());
    }

    let iteration = client
        .get_iteration()
        .iteration_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get iteration: {e}"))?;

    let name = iteration.name.clone();

    client
        .delete_iteration()
        .iteration_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete iteration: {e}"))?;

    println!("Deleted iteration {id} - {name}");
    Ok(())
}

async fn run_stories(id: i64, desc: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let mut req = client.list_iteration_stories().iteration_public_id(id);
    if desc {
        req = req.includes_description(true);
    }
    let stories = req
        .send()
        .await
        .map_err(|e| format!("Failed to list iteration stories: {e}"))?;

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
        println!("No stories in this iteration");
    }

    Ok(())
}

// --- Follower resolution ---

async fn resolve_followers(
    followers: &[String],
    client: &api::Client,
    cache_dir: &Path,
) -> Result<Vec<uuid::Uuid>, Box<dyn Error>> {
    let mut ids = Vec::with_capacity(followers.len());
    for follower in followers {
        let uuid = member::resolve_member_id(follower, client, cache_dir).await?;
        ids.push(uuid);
    }
    Ok(ids)
}
