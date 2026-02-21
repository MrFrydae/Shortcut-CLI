mod create;
mod delete;
mod get;
mod list;
mod update;
mod use_template;

pub use create::CreateArgs;
pub use update::UpdateArgs;
pub use use_template::UseArgs;

use std::error::Error;
use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::api;
use crate::output::OutputConfig;

#[derive(Args)]
pub struct TemplateArgs {
    #[command(subcommand)]
    pub action: TemplateAction,
}

#[derive(Subcommand)]
pub enum TemplateAction {
    /// List all entity templates
    List,
    /// Get an entity template by ID
    Get {
        /// The entity template UUID
        #[arg(long)]
        id: String,
    },
    /// Create a story from a template
    Use(Box<use_template::UseArgs>),
    /// Create a new entity template
    Create(Box<create::CreateArgs>),
    /// Update an entity template
    Update(Box<update::UpdateArgs>),
    /// Delete an entity template
    Delete {
        /// The entity template UUID
        #[arg(long)]
        id: String,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
}

pub async fn run(
    args: &TemplateArgs,
    client: &api::Client,
    cache_dir: PathBuf,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        TemplateAction::List => list::run(client, out).await,
        TemplateAction::Get { id } => get::run(id, client, out).await,
        TemplateAction::Use(use_args) => use_template::run(use_args, client, &cache_dir, out).await,
        TemplateAction::Create(create_args) => {
            create::run(create_args, client, &cache_dir, out).await
        }
        TemplateAction::Update(update_args) => {
            update::run(update_args, client, &cache_dir, out).await
        }
        TemplateAction::Delete { id, confirm } => delete::run(id, *confirm, client, out).await,
    }
}
