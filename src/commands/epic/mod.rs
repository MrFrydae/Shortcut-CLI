mod create;
mod delete;
mod docs;
mod get;
pub(crate) mod helpers;
mod list;
mod update;
pub mod wizard;

pub mod comment;

pub use create::CreateArgs;
pub use update::UpdateArgs;

use std::error::Error;
use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::api;
use crate::output::OutputConfig;

#[derive(Args)]
pub struct EpicArgs {
    #[command(subcommand)]
    pub action: EpicAction,
}

#[derive(Subcommand)]
pub enum EpicAction {
    /// List all epics
    List {
        /// Include epic descriptions in output
        #[arg(long, visible_alias = "descriptions")]
        desc: bool,
    },
    /// Create a new epic
    Create(Box<create::CreateArgs>),
    /// Get an epic by ID
    Get {
        /// The ID of the epic
        #[arg(long)]
        id: i64,
    },
    /// Update an epic
    Update(Box<update::UpdateArgs>),
    /// Manage comments on an epic
    Comment(comment::CommentArgs),
    /// Delete an epic
    Delete {
        /// The ID of the epic to delete
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// List documents linked to an epic
    Docs {
        /// The ID of the epic
        #[arg(long)]
        id: i64,
    },
}

pub async fn run(
    args: &EpicArgs,
    client: &api::Client,
    cache_dir: PathBuf,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        EpicAction::List { desc } => list::run(*desc, client, out).await,
        EpicAction::Create(create_args) => {
            if create_args.interactive {
                if !atty::is(atty::Stream::Stdin) {
                    return Err("Interactive mode requires a terminal".into());
                }
                let members =
                    crate::commands::member::fetch_member_choices(client, &cache_dir).await?;
                let epic_states = helpers::fetch_epic_state_names(client, &cache_dir).await?;
                let group_choices =
                    crate::commands::group::helpers::fetch_group_choices(client).await?;
                let objective_choices =
                    crate::commands::objective::helpers::fetch_objective_choices(client).await?;
                let filled = wizard::run_wizard(
                    create_args,
                    &crate::interactive::TerminalPrompter,
                    &members,
                    &epic_states,
                    &group_choices,
                    &objective_choices,
                )?;
                return create::run(&filled, client, &cache_dir, out).await;
            }
            create::run(create_args, client, &cache_dir, out).await
        }
        EpicAction::Get { id } => get::run(*id, client, &cache_dir, out).await,
        EpicAction::Update(update_args) => update::run(update_args, client, &cache_dir, out).await,
        EpicAction::Comment(args) => comment::run(args, client, &cache_dir, out).await,
        EpicAction::Delete { id, confirm } => delete::run(*id, *confirm, client, out).await,
        EpicAction::Docs { id } => docs::run(*id, client, out).await,
    }
}
