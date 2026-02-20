use std::error::Error;

use clap::{Args, Subcommand};

use crate::api;

#[derive(Args)]
pub struct ProjectArgs {
    #[command(subcommand)]
    pub action: ProjectAction,
}

#[derive(Subcommand)]
pub enum ProjectAction {
    /// List all projects
    List {
        /// Include archived projects
        #[arg(long)]
        archived: bool,
    },
    /// Create a new project
    Create(Box<CreateArgs>),
    /// Get a project by ID
    Get {
        /// The ID of the project
        #[arg(long)]
        id: i64,
    },
    /// Update a project
    Update(Box<UpdateArgs>),
    /// Delete a project
    Delete {
        /// The ID of the project to delete
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// List stories in a project
    Stories {
        /// The ID of the project
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
    /// The project name
    #[arg(long)]
    pub name: String,

    /// The ID of the team the project belongs to
    #[arg(long)]
    pub team_id: i64,

    /// The project description
    #[arg(long)]
    pub description: Option<String>,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// The abbreviation used in Story summaries (max 3 chars)
    #[arg(long)]
    pub abbreviation: Option<String>,

    /// The number of weeks per iteration
    #[arg(long)]
    pub iteration_length: Option<i64>,

    /// An external unique ID for import tracking
    #[arg(long)]
    pub external_id: Option<String>,
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// The ID of the project to update
    #[arg(long)]
    pub id: i64,

    /// The project name
    #[arg(long)]
    pub name: Option<String>,

    /// The project description
    #[arg(long)]
    pub description: Option<String>,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// The abbreviation used in Story summaries (max 3 chars)
    #[arg(long)]
    pub abbreviation: Option<String>,

    /// Whether the project is archived
    #[arg(long)]
    pub archived: Option<bool>,

    /// The ID of the team the project belongs to
    #[arg(long)]
    pub team_id: Option<i64>,

    /// The number of days before the thermometer appears
    #[arg(long)]
    pub days_to_thermometer: Option<i64>,

    /// Enable or disable thermometers in the Story summary
    #[arg(long)]
    pub show_thermometer: Option<bool>,
}

pub async fn run(args: &ProjectArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    match &args.action {
        ProjectAction::List { archived } => run_list(*archived, client).await,
        ProjectAction::Create(create_args) => run_create(create_args, client).await,
        ProjectAction::Get { id } => run_get(*id, client).await,
        ProjectAction::Update(update_args) => run_update(update_args, client).await,
        ProjectAction::Delete { id, confirm } => run_delete(*id, *confirm, client).await,
        ProjectAction::Stories { id, desc } => run_stories(*id, *desc, client).await,
    }
}

async fn run_list(include_archived: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let projects = client
        .list_projects()
        .send()
        .await
        .map_err(|e| format!("Failed to list projects: {e}"))?;

    for proj in projects.iter() {
        if !include_archived && proj.archived {
            continue;
        }
        println!(
            "{} - {} ({} stories)",
            proj.id, proj.name, proj.stats.num_stories
        );
    }
    Ok(())
}

async fn run_create(args: &CreateArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .parse::<api::types::CreateProjectName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateProjectDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let abbreviation = args
        .abbreviation
        .as_ref()
        .map(|a| a.parse::<api::types::CreateProjectAbbreviation>())
        .transpose()
        .map_err(|e| format!("Invalid abbreviation: {e}"))?;

    let external_id = args
        .external_id
        .as_ref()
        .map(|id| id.parse::<api::types::CreateProjectExternalId>())
        .transpose()
        .map_err(|e| format!("Invalid external ID: {e}"))?;

    let project = client
        .create_project()
        .body_map(|mut b| {
            b = b.name(name).team_id(args.team_id);
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if let Some(abbr) = abbreviation {
                b = b.abbreviation(Some(abbr));
            }
            if let Some(iter_len) = args.iteration_length {
                b = b.iteration_length(Some(iter_len));
            }
            if let Some(ext_id) = external_id {
                b = b.external_id(Some(ext_id));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create project: {e}"))?;

    println!(
        "Created project {} - {} ({} stories)",
        project.id, project.name, project.stats.num_stories
    );
    Ok(())
}

async fn run_get(id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let project = client
        .get_project()
        .project_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get project: {e}"))?;

    println!("{} - {}", project.id, project.name);
    println!(
        "  Description: {}",
        project.description.as_deref().unwrap_or("none")
    );
    println!(
        "  Abbreviation: {}",
        project.abbreviation.as_deref().unwrap_or("none")
    );
    println!(
        "  Color:       {}",
        project.color.as_deref().unwrap_or("none")
    );
    println!("  Team ID:     {}", project.team_id);
    println!("  Workflow ID: {}", project.workflow_id);
    println!("  Archived:    {}", project.archived);
    println!(
        "  Stats:       {} stories, {} points",
        project.stats.num_stories, project.stats.num_points
    );

    Ok(())
}

async fn run_update(args: &UpdateArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateProjectName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateProjectDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let project = client
        .update_project()
        .project_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if let Some(abbr) = &args.abbreviation {
                b = b.abbreviation(Some(abbr.clone()));
            }
            if let Some(archived) = args.archived {
                b = b.archived(Some(archived));
            }
            if let Some(team_id) = args.team_id {
                b = b.team_id(Some(team_id));
            }
            if let Some(days) = args.days_to_thermometer {
                b = b.days_to_thermometer(Some(days));
            }
            if let Some(show) = args.show_thermometer {
                b = b.show_thermometer(Some(show));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update project: {e}"))?;

    println!("Updated project {} - {}", project.id, project.name);
    Ok(())
}

async fn run_delete(id: i64, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a project is irreversible. Pass --confirm to proceed.".into());
    }

    let project = client
        .get_project()
        .project_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get project: {e}"))?;

    let name = project.name.clone();

    client
        .delete_project()
        .project_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete project: {e}"))?;

    println!("Deleted project {id} - {name}");
    Ok(())
}

async fn run_stories(id: i64, desc: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let mut req = client.list_stories().project_public_id(id);
    if desc {
        req = req.includes_description(true);
    }
    let stories = req
        .send()
        .await
        .map_err(|e| format!("Failed to list project stories: {e}"))?;

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
        println!("No stories in this project");
    }

    Ok(())
}
