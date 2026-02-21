mod create;
mod delete;
mod epics;
mod get;
mod link;
mod list;
mod unlink;
mod update;

pub use create::CreateArgs;
pub use update::UpdateArgs;

use std::error::Error;

use clap::{Args, Subcommand};

use crate::api;

#[derive(Args)]
pub struct DocArgs {
    #[command(subcommand)]
    pub action: DocAction,
}

#[derive(Subcommand)]
pub enum DocAction {
    /// List all documents
    List,
    /// Create a new document
    Create(Box<create::CreateArgs>),
    /// Get a document by ID
    Get {
        /// The document UUID
        #[arg(long)]
        id: String,
    },
    /// Update a document
    Update(Box<update::UpdateArgs>),
    /// Delete a document
    Delete {
        /// The document UUID
        #[arg(long)]
        id: String,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// Link a document to an epic
    Link {
        /// The document UUID
        #[arg(long)]
        doc_id: String,
        /// The epic ID
        #[arg(long)]
        epic_id: i64,
    },
    /// Unlink a document from an epic
    Unlink {
        /// The document UUID
        #[arg(long)]
        doc_id: String,
        /// The epic ID
        #[arg(long)]
        epic_id: i64,
    },
    /// List epics linked to a document
    Epics {
        /// The document UUID
        #[arg(long)]
        id: String,
    },
}

pub async fn run(args: &DocArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    match &args.action {
        DocAction::List => list::run(client).await,
        DocAction::Create(create_args) => create::run(create_args, client).await,
        DocAction::Get { id } => get::run(id, client).await,
        DocAction::Update(update_args) => update::run(update_args, client).await,
        DocAction::Delete { id, confirm } => delete::run(id, *confirm, client).await,
        DocAction::Link { doc_id, epic_id } => link::run(doc_id, *epic_id, client).await,
        DocAction::Unlink { doc_id, epic_id } => unlink::run(doc_id, *epic_id, client).await,
        DocAction::Epics { id } => epics::run(id, client).await,
    }
}
