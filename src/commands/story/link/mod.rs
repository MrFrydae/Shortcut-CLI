mod create;
mod delete;
mod list;

pub use create::CreateLinkArgs;

use std::error::Error;

use clap::{Args, Subcommand};

use crate::api;
use crate::output::OutputConfig;

#[derive(Args)]
pub struct LinkArgs {
    #[command(subcommand)]
    pub action: LinkAction,
}

#[derive(Subcommand)]
pub enum LinkAction {
    /// Create a relationship between two stories
    Create(create::CreateLinkArgs),
    /// List all links on a story
    List {
        /// The story ID
        #[arg(long)]
        story_id: i64,
    },
    /// Delete a story link
    Delete {
        /// The story link ID
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
}

/// Invert a verb string for displaying the object's perspective.
pub fn invert_verb(verb: &str) -> &'static str {
    match verb {
        "blocks" => "blocked by",
        "duplicates" => "duplicated by",
        "relates to" => "relates to",
        _ => "linked to",
    }
}

pub async fn run(
    args: &LinkArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        LinkAction::Create(create_args) => create::run(create_args, client, out).await,
        LinkAction::List { story_id } => list::run(*story_id, client, out).await,
        LinkAction::Delete { id, confirm } => delete::run(*id, *confirm, client, out).await,
    }
}
