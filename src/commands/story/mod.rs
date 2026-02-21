mod create;
mod delete;
mod get;
mod helpers;
mod history;
mod list;
mod update;

pub mod comment;
pub mod link;
pub mod task;

pub use create::CreateArgs;
pub use history::HistoryArgs;
pub use list::ListArgs;
pub use update::UpdateArgs;

use std::error::Error;
use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::api;
use crate::output::OutputConfig;

#[derive(Args)]
pub struct StoryArgs {
    #[command(subcommand)]
    pub action: StoryAction,
}

#[derive(Subcommand)]
pub enum StoryAction {
    /// Create a new story
    Create(Box<create::CreateArgs>),
    /// Update an existing story
    Update(Box<update::UpdateArgs>),
    /// Get a story by ID
    Get {
        /// The ID of the story
        #[arg(long)]
        id: i64,
    },
    /// List/search stories
    List(Box<list::ListArgs>),
    /// Delete a story
    Delete {
        /// The ID of the story to delete
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// Manage checklist tasks on a story
    Task(task::TaskArgs),
    /// Manage story links (relationships between stories)
    Link(link::LinkArgs),
    /// Manage comments on a story
    Comment(comment::CommentArgs),
    /// Show the change history of a story
    History(history::HistoryArgs),
}

pub async fn run(
    args: &StoryArgs,
    client: &api::Client,
    cache_dir: PathBuf,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        StoryAction::Create(create_args) => create::run(create_args, client, &cache_dir, out).await,
        StoryAction::Update(update_args) => update::run(update_args, client, &cache_dir, out).await,
        StoryAction::Get { id } => get::run(*id, client, &cache_dir, out).await,
        StoryAction::List(list_args) => list::run(list_args, client, &cache_dir, out).await,
        StoryAction::Delete { id, confirm } => delete::run(*id, *confirm, client, out).await,
        StoryAction::Task(task_args) => task::run(task_args, client, out).await,
        StoryAction::Link(link_args) => link::run(link_args, client, out).await,
        StoryAction::Comment(args) => comment::run(args, client, &cache_dir, out).await,
        StoryAction::History(history_args) => {
            history::run(history_args, client, &cache_dir, out).await
        }
    }
}
