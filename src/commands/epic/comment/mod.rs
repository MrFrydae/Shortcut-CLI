mod add;
mod delete;
mod get;
mod helpers;
mod list;
mod update;

use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};

use crate::api;

#[derive(Args)]
pub struct CommentArgs {
    #[command(subcommand)]
    pub action: CommentAction,
}

#[derive(Subcommand)]
pub enum CommentAction {
    /// List all comments on an epic
    List {
        /// The epic ID
        #[arg(long)]
        epic_id: i64,
    },
    /// Add a comment to an epic
    Add {
        /// The epic ID
        #[arg(long)]
        epic_id: i64,
        /// Comment text
        #[arg(long)]
        text: Option<String>,
        /// Read comment text from a file
        #[arg(long)]
        text_file: Option<PathBuf>,
    },
    /// Get a single comment
    Get {
        /// The epic ID
        #[arg(long)]
        epic_id: i64,
        /// The comment ID
        #[arg(long)]
        id: i64,
    },
    /// Update a comment's text
    Update {
        /// The epic ID
        #[arg(long)]
        epic_id: i64,
        /// The comment ID
        #[arg(long)]
        id: i64,
        /// New comment text
        #[arg(long)]
        text: String,
    },
    /// Delete a comment
    Delete {
        /// The epic ID
        #[arg(long)]
        epic_id: i64,
        /// The comment ID
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
}

pub async fn run(
    args: &CommentArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        CommentAction::List { epic_id } => list::run(*epic_id, client, cache_dir).await,
        CommentAction::Add {
            epic_id,
            text,
            text_file,
        } => add::run(*epic_id, text.as_deref(), text_file.as_deref(), client).await,
        CommentAction::Get { epic_id, id } => get::run(*epic_id, *id, client, cache_dir).await,
        CommentAction::Update { epic_id, id, text } => {
            update::run(*epic_id, *id, text, client).await
        }
        CommentAction::Delete {
            epic_id,
            id,
            confirm,
        } => delete::run(*epic_id, *id, *confirm, client).await,
    }
}
