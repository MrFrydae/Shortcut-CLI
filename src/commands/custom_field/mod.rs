mod get;
pub mod helpers;
mod list;

use std::error::Error;
use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::api;
use crate::output::OutputConfig;

pub use helpers::{resolve_custom_field_names, resolve_custom_field_value};

#[derive(Args)]
pub struct CustomFieldArgs {
    #[command(subcommand)]
    pub action: CustomFieldAction,
}

#[derive(Subcommand)]
pub enum CustomFieldAction {
    /// List all custom fields
    List,
    /// Get a custom field by ID
    Get {
        /// The UUID of the custom field
        #[arg(long)]
        id: String,
    },
}

pub async fn run(
    args: &CustomFieldArgs,
    client: &api::Client,
    cache_dir: PathBuf,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        CustomFieldAction::List => list::run(client, &cache_dir, out).await,
        CustomFieldAction::Get { id } => get::run(id, client, out).await,
    }
}
