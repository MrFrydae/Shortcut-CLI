mod create;
mod delete;
mod get;
mod list;
mod stories;
mod update;

pub use create::CreateArgs;
pub use update::UpdateArgs;

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
    Create(Box<create::CreateArgs>),
    /// Get a project by ID
    Get {
        /// The ID of the project
        #[arg(long)]
        id: i64,
    },
    /// Update a project
    Update(Box<update::UpdateArgs>),
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

pub async fn run(args: &ProjectArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    match &args.action {
        ProjectAction::List { archived } => list::run(*archived, client).await,
        ProjectAction::Create(create_args) => create::run(create_args, client).await,
        ProjectAction::Get { id } => get::run(*id, client).await,
        ProjectAction::Update(update_args) => update::run(update_args, client).await,
        ProjectAction::Delete { id, confirm } => delete::run(*id, *confirm, client).await,
        ProjectAction::Stories { id, desc } => stories::run(*id, *desc, client).await,
    }
}
