mod create;
mod delete;
mod docs;
mod get;
mod helpers;
mod list;
mod update;

pub mod comment;

pub use create::CreateArgs;
pub use update::UpdateArgs;

use std::error::Error;
use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::api;
use crate::output::OutputConfig;

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
    /// Create a new epic
    Create(Box<create::CreateArgs>),
    /// Get an epic by ID
    Get {
        /// The ID of the epic
        #[arg(long)]
        id: i64,
    },
    /// Update an epic
    Update(Box<update::UpdateArgs>),
    /// Manage comments on an epic
    Comment(comment::CommentArgs),
    /// Delete an epic
    Delete {
        /// The ID of the epic to delete
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// List documents linked to an epic
    Docs {
        /// The ID of the epic
        #[arg(long)]
        id: i64,
    },
}

pub async fn run(
    args: &EpicArgs,
    client: &api::Client,
    cache_dir: PathBuf,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        EpicAction::List { desc } => list::run(*desc, client, out).await,
        EpicAction::Create(create_args) => create::run(create_args, client, &cache_dir, out).await,
        EpicAction::Get { id } => get::run(*id, client, &cache_dir, out).await,
        EpicAction::Update(update_args) => update::run(update_args, client, &cache_dir, out).await,
        EpicAction::Comment(args) => comment::run(args, client, &cache_dir, out).await,
        EpicAction::Delete { id, confirm } => delete::run(*id, *confirm, client, out).await,
        EpicAction::Docs { id } => docs::run(*id, client, out).await,
    }
}
