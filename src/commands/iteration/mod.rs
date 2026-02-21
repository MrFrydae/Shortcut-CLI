mod create;
mod delete;
mod get;
mod helpers;
mod list;
mod stories;
mod update;

use std::error::Error;
use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::api;

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
    Create(Box<create::CreateArgs>),
    /// Get an iteration by ID
    Get {
        /// The ID of the iteration
        #[arg(long)]
        id: i64,
    },
    /// Update an iteration
    Update(Box<update::UpdateArgs>),
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

pub async fn run(
    args: &IterationArgs,
    client: &api::Client,
    cache_dir: PathBuf,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        IterationAction::List { state } => list::run(state.as_deref(), client).await,
        IterationAction::Create(create_args) => create::run(create_args, client, &cache_dir).await,
        IterationAction::Get { id } => get::run(*id, client).await,
        IterationAction::Update(update_args) => update::run(update_args, client, &cache_dir).await,
        IterationAction::Delete { id, confirm } => delete::run(*id, *confirm, client).await,
        IterationAction::Stories { id, desc } => stories::run(*id, *desc, client).await,
    }
}
