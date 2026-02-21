mod create;
mod delete;
mod get;
mod list;
mod milestones;
mod objectives;
mod update;

pub use create::CreateArgs;
pub use update::UpdateArgs;

use std::error::Error;

use clap::{Args, Subcommand};

use crate::api;

#[derive(Args)]
pub struct CategoryArgs {
    #[command(subcommand)]
    pub action: CategoryAction,
}

#[derive(Subcommand)]
pub enum CategoryAction {
    /// List all categories
    List {
        /// Include archived categories
        #[arg(long)]
        archived: bool,
    },
    /// Create a new category
    Create(Box<create::CreateArgs>),
    /// Get a category by ID
    Get {
        /// The ID of the category
        #[arg(long)]
        id: i64,
    },
    /// Update a category
    Update(Box<update::UpdateArgs>),
    /// Delete a category
    Delete {
        /// The ID of the category to delete
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// List milestones associated with a category
    Milestones {
        /// The ID of the category
        #[arg(long)]
        id: i64,
    },
    /// List objectives associated with a category
    Objectives {
        /// The ID of the category
        #[arg(long)]
        id: i64,
        /// Include objective descriptions in output
        #[arg(long, visible_alias = "descriptions")]
        desc: bool,
    },
}

pub async fn run(args: &CategoryArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    match &args.action {
        CategoryAction::List { archived } => list::run(*archived, client).await,
        CategoryAction::Create(create_args) => create::run(create_args, client).await,
        CategoryAction::Get { id } => get::run(*id, client).await,
        CategoryAction::Update(update_args) => update::run(update_args, client).await,
        CategoryAction::Delete { id, confirm } => delete::run(*id, *confirm, client).await,
        CategoryAction::Milestones { id } => milestones::run(*id, client).await,
        CategoryAction::Objectives { id, desc } => objectives::run(*id, *desc, client).await,
    }
}
