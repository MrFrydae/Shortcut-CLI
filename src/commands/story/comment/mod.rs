mod add;
mod delete;
mod get;
mod helpers;
mod list;
mod react;
mod unreact;
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
    /// List all comments on a story
    List {
        /// The story ID
        #[arg(long)]
        story_id: i64,
    },
    /// Add a comment to a story
    Add {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// Comment text
        #[arg(long)]
        text: Option<String>,
        /// Read comment text from a file
        #[arg(long)]
        text_file: Option<PathBuf>,
    },
    /// Get a single comment
    Get {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The comment ID
        #[arg(long)]
        id: i64,
    },
    /// Update a comment's text
    Update {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The comment ID
        #[arg(long)]
        id: i64,
        /// New comment text
        #[arg(long)]
        text: String,
    },
    /// Delete a comment
    Delete {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The comment ID
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// Add a reaction to a comment
    React {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The comment ID
        #[arg(long)]
        comment_id: i64,
        /// The emoji name (e.g. thumbsup)
        #[arg(long)]
        emoji: String,
    },
    /// Remove a reaction from a comment
    Unreact {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The comment ID
        #[arg(long)]
        comment_id: i64,
        /// The emoji name (e.g. thumbsup)
        #[arg(long)]
        emoji: String,
    },
}

pub async fn run(
    args: &CommentArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        CommentAction::List { story_id } => list::run(*story_id, client, cache_dir).await,
        CommentAction::Add {
            story_id,
            text,
            text_file,
        } => add::run(*story_id, text.as_deref(), text_file.as_deref(), client).await,
        CommentAction::Get { story_id, id } => get::run(*story_id, *id, client, cache_dir).await,
        CommentAction::Update { story_id, id, text } => {
            update::run(*story_id, *id, text, client).await
        }
        CommentAction::Delete {
            story_id,
            id,
            confirm,
        } => delete::run(*story_id, *id, *confirm, client).await,
        CommentAction::React {
            story_id,
            comment_id,
            emoji,
        } => react::run(*story_id, *comment_id, emoji, client).await,
        CommentAction::Unreact {
            story_id,
            comment_id,
            emoji,
        } => unreact::run(*story_id, *comment_id, emoji, client).await,
    }
}
