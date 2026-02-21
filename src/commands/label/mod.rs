mod create;
mod delete;
mod epics;
mod get;
pub mod helpers;
mod list;
mod stories;
mod update;

use std::error::Error;
use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::api;

pub use helpers::resolve_label_id;

#[derive(Args)]
pub struct LabelArgs {
    #[command(subcommand)]
    pub action: LabelAction,
}

#[derive(Subcommand)]
pub enum LabelAction {
    /// List all labels
    List {
        /// Include label descriptions in output
        #[arg(long, visible_alias = "descriptions")]
        desc: bool,
    },
    /// Create a new label
    Create(Box<create::CreateArgs>),
    /// Get a label by ID
    Get {
        /// The ID of the label
        #[arg(long)]
        id: i64,
    },
    /// Update a label
    Update(Box<update::UpdateArgs>),
    /// Delete a label
    Delete {
        /// The ID of the label to delete
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// List stories with a label
    Stories {
        /// The ID of the label
        #[arg(long)]
        id: i64,
        /// Include story descriptions in output
        #[arg(long, visible_alias = "descriptions")]
        desc: bool,
    },
    /// List epics with a label
    Epics {
        /// The ID of the label
        #[arg(long)]
        id: i64,
        /// Include epic descriptions in output
        #[arg(long, visible_alias = "descriptions")]
        desc: bool,
    },
}

pub async fn run(
    args: &LabelArgs,
    client: &api::Client,
    cache_dir: PathBuf,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        LabelAction::List { desc } => list::run(*desc, client, &cache_dir).await,
        LabelAction::Create(create_args) => create::run(create_args, client, &cache_dir).await,
        LabelAction::Get { id } => get::run(*id, client).await,
        LabelAction::Update(update_args) => update::run(update_args, client).await,
        LabelAction::Delete { id, confirm } => delete::run(*id, *confirm, client).await,
        LabelAction::Stories { id, desc } => stories::run(*id, *desc, client).await,
        LabelAction::Epics { id, desc } => epics::run(*id, *desc, client).await,
    }
}
