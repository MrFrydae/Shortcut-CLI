//! `shortcut file`: upload and manage files in the Shortcut workspace.

mod delete;
mod get;
pub mod helpers;
mod list;
mod update;
pub mod upload;

pub use update::UpdateArgs;
pub use upload::UploadArgs;

use std::error::Error;

use clap::{Args, Subcommand};

use crate::api;
use crate::output::OutputConfig;

#[derive(Args)]
pub struct FileArgs {
    #[command(subcommand)]
    pub action: FileAction,
}

#[derive(Subcommand)]
pub enum FileAction {
    /// Upload files, optionally attaching them to a story
    #[command(
        after_help = "Files are sent in batches of 4 (the documented per-request limit).\n\nExamples:\n  shortcut file upload mockup.png\n  shortcut file upload --story-id 123 a.png b.png\n  shortcut file upload --story-id 123 --markdown a.png\n  shortcut file upload --story-id 123 --embed a.png b.png"
    )]
    Upload(Box<upload::UploadArgs>),
    /// List all files in the workspace
    List,
    /// Get a file by ID
    Get {
        /// The ID of the file
        #[arg(long)]
        id: i64,
    },
    /// Update a file's name or description
    Update(Box<update::UpdateArgs>),
    /// Delete a file
    Delete {
        /// The ID of the file to delete
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
}

pub async fn run(
    args: &FileArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        FileAction::Upload(upload_args) => upload::run(upload_args, client, out).await,
        FileAction::List => list::run(client, out).await,
        FileAction::Get { id } => get::run(*id, client, out).await,
        FileAction::Update(update_args) => update::run(update_args, client, out).await,
        FileAction::Delete { id, confirm } => delete::run(*id, *confirm, client, out).await,
    }
}
