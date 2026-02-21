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

pub async fn run(args: &SearchArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    match &args.action {
        SearchAction::All(q) => all::run(q, client).await,
        SearchAction::Stories(q) => stories::run(q, client).await,
        SearchAction::Epics(q) => epics::run(q, client).await,
        SearchAction::Iterations(q) => iterations::run(q, client).await,
        SearchAction::Milestones(q) => milestones::run(q, client).await,
        SearchAction::Objectives(q) => objectives::run(q, client).await,
        SearchAction::Documents(q) => documents::run(q, client).await,
    }
}
