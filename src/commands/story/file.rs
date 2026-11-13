//! `shortcut story file`: story-scoped alias for `shortcut file upload`.

use std::error::Error;
use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::api;
use crate::commands::file::upload::{UploadOptions, upload};
use crate::output::OutputConfig;

#[derive(Args)]
pub struct StoryFileArgs {
    #[command(subcommand)]
    pub action: StoryFileAction,
}

#[derive(Subcommand)]
pub enum StoryFileAction {
    /// Upload files and attach them to a story (alias for `file upload --story-id`)
    #[command(
        after_help = "Examples:\n  shortcut story file upload --story-id 123 a.png b.png\n  shortcut story file upload --story-id 123 --embed mockup.png"
    )]
    Upload {
        /// The story to attach the files to
        #[arg(long)]
        story_id: i64,
        /// Files to upload
        #[arg(required = true, value_name = "PATH")]
        paths: Vec<PathBuf>,
        /// Print a ready-to-paste markdown image/link per file instead of the summary
        #[arg(long)]
        markdown: bool,
        /// Append markdown images/links for the uploaded files to the story description
        #[arg(long)]
        embed: bool,
    },
}

pub async fn run(
    args: &StoryFileArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        StoryFileAction::Upload {
            story_id,
            paths,
            markdown,
            embed,
        } => {
            upload(
                &UploadOptions {
                    paths,
                    story_id: Some(*story_id),
                    markdown: *markdown,
                    embed: *embed,
                },
                client,
                out,
            )
            .await
        }
    }
}
