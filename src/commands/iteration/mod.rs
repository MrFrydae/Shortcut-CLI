mod create;
mod delete;
mod get;
pub(crate) mod helpers;
mod list;
mod stories;
mod update;
pub mod wizard;

pub use create::CreateArgs;

use std::error::Error;
use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::api;
use crate::output::OutputConfig;

#[derive(Args)]
pub struct IterationArgs {
    #[command(subcommand)]
    pub action: IterationAction,
}

#[derive(Subcommand)]
pub enum IterationAction {
    /// List all iterations
    List {
        /// Filter by status (e.g. "started", "unstarted", "done")
        #[arg(long)]
        state: Option<String>,
    },
    /// Create a new iteration
    Create(Box<create::CreateArgs>),
    /// Get an iteration by ID
    Get {
        /// The ID of the iteration
        #[arg(long)]
        id: i64,
    },
    /// Update an iteration
    Update(Box<update::UpdateArgs>),
    /// Delete an iteration
    Delete {
        /// The ID of the iteration to delete
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// List stories in an iteration
    Stories {
        /// The ID of the iteration
        #[arg(long)]
        id: i64,
        /// Include story descriptions in output
        #[arg(long, visible_alias = "descriptions")]
        desc: bool,
    },
}

pub async fn run(
    args: &IterationArgs,
    client: &api::Client,
    cache_dir: PathBuf,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        IterationAction::List { state } => list::run(state.as_deref(), client, out).await,
        IterationAction::Create(create_args) => {
            if create_args.interactive {
                if !atty::is(atty::Stream::Stdin) {
                    return Err("Interactive mode requires a terminal".into());
                }
                let members =
                    crate::commands::member::fetch_member_choices(client, &cache_dir).await?;
                let filled = wizard::run_wizard(
                    create_args,
                    &crate::interactive::TerminalPrompter,
                    &members,
                )?;
                return create::run(&filled, client, &cache_dir, out).await;
            }
            create::run(create_args, client, &cache_dir, out).await
        }
        IterationAction::Get { id } => get::run(*id, client, out).await,
        IterationAction::Update(update_args) => {
            update::run(update_args, client, &cache_dir, out).await
        }
        IterationAction::Delete { id, confirm } => delete::run(*id, *confirm, client, out).await,
        IterationAction::Stories { id, desc } => stories::run(*id, *desc, client, out).await,
    }
}
