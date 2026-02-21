mod add;
mod check;
mod delete;
mod get;
mod list;
mod update;

pub use add::AddArgs;
pub use update::UpdateTaskArgs;

use std::error::Error;

use clap::{Args, Subcommand};

use crate::api;

#[derive(Args)]
pub struct TaskArgs {
    #[command(subcommand)]
    pub action: TaskAction,
}

#[derive(Subcommand)]
pub enum TaskAction {
    /// Add one or more tasks to a story
    Add(add::AddArgs),
    /// List all tasks on a story
    List {
        /// The story ID
        #[arg(long)]
        story_id: i64,
    },
    /// Get a single task
    Get {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The task ID
        #[arg(long)]
        id: i64,
    },
    /// Mark a task as complete
    Check {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The task ID
        #[arg(long)]
        id: i64,
    },
    /// Mark a task as incomplete
    Uncheck {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The task ID
        #[arg(long)]
        id: i64,
    },
    /// Update a task's description
    Update(update::UpdateTaskArgs),
    /// Delete a task
    Delete {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The task ID
        #[arg(long)]
        id: i64,
    },
}

pub async fn run(args: &TaskArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    match &args.action {
        TaskAction::Add(add_args) => add::run(add_args, client).await,
        TaskAction::List { story_id } => list::run(*story_id, client).await,
        TaskAction::Get { story_id, id } => get::run(*story_id, *id, client).await,
        TaskAction::Check { story_id, id } => check::run(*story_id, *id, true, client).await,
        TaskAction::Uncheck { story_id, id } => check::run(*story_id, *id, false, client).await,
        TaskAction::Update(update_args) => update::run(update_args, client).await,
        TaskAction::Delete { story_id, id } => delete::run(*story_id, *id, client).await,
    }
}
