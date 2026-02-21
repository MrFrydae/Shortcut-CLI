mod create;
mod get;
pub mod helpers;
mod list;
mod stories;
mod update;

pub use create::CreateArgs;
pub use update::UpdateArgs;

use std::error::Error;
use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::api;

#[derive(Args)]
pub struct GroupArgs {
    #[command(subcommand)]
    pub action: GroupAction,
}

#[derive(Subcommand)]
pub enum GroupAction {
    /// List all groups
    List {
        /// Include archived groups
        #[arg(long)]
        archived: bool,
    },
    /// Create a new group
    Create(Box<create::CreateArgs>),
    /// Get a group by ID
    Get {
        /// Group @mention_name or UUID
        #[arg(long)]
        id: String,
    },
    /// Update a group
    Update(Box<update::UpdateArgs>),
    /// List stories for a group
    Stories {
        /// Group @mention_name or UUID
        #[arg(long)]
        id: String,
        /// Maximum number of stories to return
        #[arg(long)]
        limit: Option<i64>,
        /// Offset for pagination
        #[arg(long)]
        offset: Option<i64>,
        /// Include story descriptions in output
        #[arg(long, visible_alias = "descriptions")]
        desc: bool,
    },
}

pub async fn run(
    args: &GroupArgs,
    client: &api::Client,
    cache_dir: PathBuf,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        GroupAction::List { archived } => list::run(*archived, client, &cache_dir).await,
        GroupAction::Create(create_args) => create::run(create_args, client, &cache_dir).await,
        GroupAction::Get { id } => get::run(id, client, &cache_dir).await,
        GroupAction::Update(update_args) => update::run(update_args, client, &cache_dir).await,
        GroupAction::Stories {
            id,
            limit,
            offset,
            desc,
        } => stories::run(id, *limit, *offset, *desc, client, &cache_dir).await,
    }
}
