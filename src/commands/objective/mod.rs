mod create;
mod delete;
mod epics;
mod get;
mod helpers;
mod list;
mod update;

pub use create::CreateArgs;
pub use update::UpdateArgs;

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
    Create(Box<create::CreateArgs>),
    /// Get an objective by ID
    Get {
        /// The ID of the objective
        #[arg(long)]
        id: i64,
    },
    /// Update an objective
    Update(Box<update::UpdateArgs>),
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

pub async fn run(args: &ObjectiveArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    match &args.action {
        ObjectiveAction::List { archived } => list::run(*archived, client).await,
        ObjectiveAction::Create(create_args) => create::run(create_args, client).await,
        ObjectiveAction::Get { id } => get::run(*id, client).await,
        ObjectiveAction::Update(update_args) => update::run(update_args, client).await,
        ObjectiveAction::Delete { id, confirm } => delete::run(*id, *confirm, client).await,
        ObjectiveAction::Epics { id, desc } => epics::run(*id, *desc, client).await,
    }
}
