use std::error::Error;

use clap::{Args, Subcommand};

use crate::api;

#[derive(Args)]
pub struct ObjectiveArgs {
    #[command(subcommand)]
    pub action: ObjectiveAction,
}

#[derive(Subcommand)]
pub enum ObjectiveAction {
    /// List all objectives
    List {
        /// Include archived objectives
        #[arg(long)]
        archived: bool,
    },
    /// Create a new objective
    Create(Box<CreateArgs>),
    /// Get an objective by ID
    Get {
        /// The ID of the objective
        #[arg(long)]
        id: i64,
    },
    /// Update an objective
    Update(Box<UpdateArgs>),
    /// Delete an objective
    Delete {
        /// The ID of the objective to delete
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// List epics associated with an objective
    Epics {
        /// The ID of the objective
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
    /// The objective name
    #[arg(long)]
    pub name: String,

    /// The objective description
    #[arg(long)]
    pub description: Option<String>,

    /// The state ("to do", "in progress", "done")
    #[arg(long)]
    pub state: Option<String>,

    /// Category names (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub categories: Vec<String>,
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// The ID of the objective to update
    #[arg(long)]
    pub id: i64,

    /// The objective name
    #[arg(long)]
    pub name: Option<String>,

    /// The objective description
    #[arg(long)]
    pub description: Option<String>,

    /// The state ("to do", "in progress", "done")
    #[arg(long)]
    pub state: Option<String>,

    /// Whether the objective is archived
    #[arg(long)]
    pub archived: Option<bool>,

    /// Category names (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub categories: Vec<String>,
}

pub async fn run(args: &ObjectiveArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    match &args.action {
        ObjectiveAction::List { archived } => run_list(*archived, client).await,
        ObjectiveAction::Create(create_args) => run_create(create_args, client).await,
        ObjectiveAction::Get { id } => run_get(*id, client).await,
        ObjectiveAction::Update(update_args) => run_update(update_args, client).await,
        ObjectiveAction::Delete { id, confirm } => run_delete(*id, *confirm, client).await,
        ObjectiveAction::Epics { id, desc } => run_epics(*id, *desc, client).await,
    }
}

async fn run_list(include_archived: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let objectives = client
        .list_objectives()
        .send()
        .await
        .map_err(|e| format!("Failed to list objectives: {e}"))?;

    for obj in objectives.iter() {
        if !include_archived && obj.archived {
            continue;
        }
        println!("{} - {} ({})", obj.id, obj.name, obj.state);
    }
    Ok(())
}

async fn run_create(args: &CreateArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .parse::<api::types::CreateObjectiveName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateObjectiveDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let state = args
        .state
        .as_ref()
        .map(|s| s.parse::<api::types::CreateObjectiveState>())
        .transpose()
        .map_err(|e| format!("Invalid state: {e}"))?;

    let categories = build_categories(&args.categories)?;

    let objective = client
        .create_objective()
        .body_map(|mut b| {
            b = b.name(name);
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(state) = state {
                b = b.state(Some(state));
            }
            if !categories.is_empty() {
                b = b.categories(categories);
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create objective: {e}"))?;

    println!(
        "Created objective {} - {} ({})",
        objective.id, objective.name, objective.state
    );
    Ok(())
}

async fn run_get(id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let objective = client
        .get_objective()
        .objective_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get objective: {e}"))?;

    println!("{} - {}", objective.id, objective.name);
    println!("  State:       {}", objective.state);
    println!("  Archived:    {}", objective.archived);
    println!("  Started:     {}", objective.started);
    println!("  Completed:   {}", objective.completed);

    if !objective.categories.is_empty() {
        let names: Vec<&str> = objective
            .categories
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        println!("  Categories:  {}", names.join(", "));
    }

    let stats = &objective.stats;
    println!("  Documents:   {}", stats.num_related_documents);
    if let Some(cycle) = stats.average_cycle_time {
        let cycle_days = cycle as f64 / 86400.0;
        println!("  Avg Cycle:   {cycle_days:.1} days");
    }
    if let Some(lead) = stats.average_lead_time {
        let lead_days = lead as f64 / 86400.0;
        println!("  Avg Lead:    {lead_days:.1} days");
    }

    if !objective.description.is_empty() {
        println!("  Description: {}", objective.description);
    }

    // Show associated epics
    let epics = client
        .list_objective_epics()
        .objective_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list objective epics: {e}"))?;

    if !epics.is_empty() {
        println!("  Epics:");
        for epic in epics.iter() {
            println!("    {} - {}", epic.id, epic.name);
        }
    }

    Ok(())
}

async fn run_update(args: &UpdateArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateObjectiveName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateObjectiveDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let state = args
        .state
        .as_ref()
        .map(|s| s.parse::<api::types::UpdateObjectiveState>())
        .transpose()
        .map_err(|e| format!("Invalid state: {e}"))?;

    let categories = build_categories(&args.categories)?;

    let objective = client
        .update_objective()
        .objective_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(state) = state {
                b = b.state(Some(state));
            }
            if let Some(archived) = args.archived {
                b = b.archived(Some(archived));
            }
            if !categories.is_empty() {
                b = b.categories(categories);
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update objective: {e}"))?;

    println!("Updated objective {} - {}", objective.id, objective.name);
    Ok(())
}

async fn run_delete(id: i64, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting an objective is irreversible. Pass --confirm to proceed.".into());
    }

    let objective = client
        .get_objective()
        .objective_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get objective: {e}"))?;

    let name = objective.name.clone();

    client
        .delete_objective()
        .objective_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete objective: {e}"))?;

    println!("Deleted objective {id} - {name}");
    Ok(())
}

async fn run_epics(id: i64, desc: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let epics = client
        .list_objective_epics()
        .objective_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list objective epics: {e}"))?;

    for epic in epics.iter() {
        println!("{} - {}", epic.id, epic.name);
        if desc && let Some(d) = &epic.description {
            println!("  {d}");
        }
    }

    if epics.is_empty() {
        println!("No epics for this objective");
    }

    Ok(())
}

fn build_categories(names: &[String]) -> Result<Vec<api::types::CreateCategoryParams>, String> {
    names
        .iter()
        .map(|n| {
            Ok(api::types::CreateCategoryParams {
                name: n
                    .parse()
                    .map_err(|e| format!("Invalid category name: {e}"))?,
                color: None,
                external_id: None,
            })
        })
        .collect()
}
