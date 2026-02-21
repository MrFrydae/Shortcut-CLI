mod all;
mod documents;
mod epics;
mod helpers;
mod iterations;
mod milestones;
mod objectives;
mod stories;

use std::error::Error;

use clap::{Args, Subcommand};

use crate::api;
use crate::output::OutputConfig;

#[derive(Args)]
pub struct SearchArgs {
    #[command(subcommand)]
    pub action: SearchAction,
}

#[derive(Subcommand)]
pub enum SearchAction {
    /// Search across all entity types (stories, epics, iterations, objectives)
    All(SearchQueryArgs),
    /// Search stories
    Stories(SearchQueryArgs),
    /// Search epics
    Epics(SearchQueryArgs),
    /// Search iterations
    Iterations(SearchQueryArgs),
    /// Search milestones (objectives)
    Milestones(SearchQueryArgs),
    /// Search objectives
    Objectives(SearchQueryArgs),
    /// Search documents (by title)
    Documents(SearchQueryArgs),
}

#[derive(Args)]
pub struct SearchQueryArgs {
    /// The search query string
    pub query: String,

    /// Results per page (1-250)
    #[arg(long, default_value = "25")]
    pub page_size: i64,

    /// Next page cursor token
    #[arg(long)]
    pub next: Option<String>,

    /// Include descriptions in output
    #[arg(long, visible_alias = "descriptions")]
    pub desc: bool,
}

pub async fn run(
    args: &SearchArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        SearchAction::All(q) => all::run(q, client, out).await,
        SearchAction::Stories(q) => stories::run(q, client, out).await,
        SearchAction::Epics(q) => epics::run(q, client, out).await,
        SearchAction::Iterations(q) => iterations::run(q, client, out).await,
        SearchAction::Milestones(q) => milestones::run(q, client, out).await,
        SearchAction::Objectives(q) => objectives::run(q, client, out).await,
        SearchAction::Documents(q) => documents::run(q, client, out).await,
    }
}
