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
        SearchAction::All(q) => run_all(q, client).await,
        SearchAction::Stories(q) => run_stories(q, client).await,
        SearchAction::Epics(q) => run_epics(q, client).await,
        SearchAction::Iterations(q) => run_iterations(q, client).await,
        SearchAction::Milestones(q) => run_milestones(q, client).await,
        SearchAction::Objectives(q) => run_objectives(q, client).await,
        SearchAction::Documents(q) => run_documents(q, client).await,
    }
}

async fn run_all(args: &SearchQueryArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let query = args
        .query
        .parse::<api::types::SearchQuery>()
        .map_err(|e| format!("Invalid query: {e}"))?;

    let detail = if args.desc {
        api::types::SearchDetail::Full
    } else {
        api::types::SearchDetail::Slim
    };

    let mut req = client
        .search()
        .query(query)
        .page_size(args.page_size)
        .detail(detail);
    if let Some(next) = &args.next {
        req = req.next(next.clone());
    }

    let results = req
        .send()
        .await
        .map_err(|e| format!("Failed to search: {e}"))?;

    let mut total = 0;

    if let Some(stories) = &results.stories {
        let count = stories.data.len();
        if count > 0 {
            let label = if count == 1 { "result" } else { "results" };
            println!("Stories ({count} {label}):");
            for story in &stories.data {
                println!(
                    "  {} - {} ({}, state_id: {})",
                    story.id, story.name, story.story_type, story.workflow_state_id
                );
                if args.desc
                    && let Some(d) = &story.description
                {
                    println!("    {d}");
                }
            }
            println!();
            total += count;
        }
    }

    if let Some(epics) = &results.epics {
        let count = epics.data.len();
        if count > 0 {
            let label = if count == 1 { "result" } else { "results" };
            println!("Epics ({count} {label}):");
            for epic in &epics.data {
                println!("  {} - {}", epic.id, epic.name);
                if args.desc
                    && let Some(d) = &epic.description
                {
                    println!("    {d}");
                }
            }
            println!();
            total += count;
        }
    }

    if let Some(iterations) = &results.iterations {
        let count = iterations.data.len();
        if count > 0 {
            let label = if count == 1 { "result" } else { "results" };
            println!("Iterations ({count} {label}):");
            for iter in &iterations.data {
                println!(
                    "  {} - {} ({}, {} \u{2192} {})",
                    iter.id, iter.name, iter.status, iter.start_date, iter.end_date
                );
            }
            println!();
            total += count;
        }
    }

    if let Some(milestones) = &results.milestones {
        let count = milestones.data.len();
        if count > 0 {
            let label = if count == 1 { "result" } else { "results" };
            println!("Objectives ({count} {label}):");
            for obj in &milestones.data {
                println!("  {} - {} ({})", obj.id, obj.name, obj.state);
                if args.desc
                    && let Some(d) = &obj.description
                {
                    println!("    {d}");
                }
            }
            println!();
            total += count;
        }
    }

    if total == 0 {
        println!("No results found");
    } else {
        let label = if total == 1 { "result" } else { "results" };
        println!("Showing {total} {label}.");
    }

    Ok(())
}

async fn run_stories(args: &SearchQueryArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let query = args
        .query
        .parse::<api::types::SearchStoriesQuery>()
        .map_err(|e| format!("Invalid query: {e}"))?;

    let detail = if args.desc {
        api::types::SearchStoriesDetail::Full
    } else {
        api::types::SearchStoriesDetail::Slim
    };

    let mut req = client
        .search_stories()
        .query(query)
        .page_size(args.page_size)
        .detail(detail);
    if let Some(next) = &args.next {
        req = req.next(next.clone());
    }

    let results = req
        .send()
        .await
        .map_err(|e| format!("Failed to search stories: {e}"))?;

    for story in &results.data {
        println!(
            "{} - {} ({}, state_id: {})",
            story.id, story.name, story.story_type, story.workflow_state_id
        );
        if args.desc
            && let Some(d) = &story.description
        {
            println!("  {d}");
        }
    }

    if results.data.is_empty() {
        println!("No stories found");
    }

    print_pagination(results.data.len(), results.total, results.next.as_deref());

    Ok(())
}

async fn run_epics(args: &SearchQueryArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let query = args
        .query
        .parse::<api::types::SearchEpicsQuery>()
        .map_err(|e| format!("Invalid query: {e}"))?;

    let detail = if args.desc {
        api::types::SearchEpicsDetail::Full
    } else {
        api::types::SearchEpicsDetail::Slim
    };

    let mut req = client
        .search_epics()
        .query(query)
        .page_size(args.page_size)
        .detail(detail);
    if let Some(next) = &args.next {
        req = req.next(next.clone());
    }

    let results = req
        .send()
        .await
        .map_err(|e| format!("Failed to search epics: {e}"))?;

    for epic in &results.data {
        println!("{} - {}", epic.id, epic.name);
        if args.desc
            && let Some(d) = &epic.description
        {
            println!("  {d}");
        }
    }

    if results.data.is_empty() {
        println!("No epics found");
    }

    print_pagination(results.data.len(), results.total, results.next.as_deref());

    Ok(())
}

async fn run_iterations(
    args: &SearchQueryArgs,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    let query = args
        .query
        .parse::<api::types::SearchIterationsQuery>()
        .map_err(|e| format!("Invalid query: {e}"))?;

    let detail = if args.desc {
        api::types::SearchIterationsDetail::Full
    } else {
        api::types::SearchIterationsDetail::Slim
    };

    let mut req = client
        .search_iterations()
        .query(query)
        .page_size(args.page_size)
        .detail(detail);
    if let Some(next) = &args.next {
        req = req.next(next.clone());
    }

    let results = req
        .send()
        .await
        .map_err(|e| format!("Failed to search iterations: {e}"))?;

    for iter in &results.data {
        println!(
            "{} - {} ({}, {} \u{2192} {})",
            iter.id, iter.name, iter.status, iter.start_date, iter.end_date
        );
    }

    if results.data.is_empty() {
        println!("No iterations found");
    }

    print_pagination(results.data.len(), results.total, results.next.as_deref());

    Ok(())
}

async fn run_milestones(
    args: &SearchQueryArgs,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    let query = args
        .query
        .parse::<api::types::SearchMilestonesQuery>()
        .map_err(|e| format!("Invalid query: {e}"))?;

    let detail = if args.desc {
        api::types::SearchMilestonesDetail::Full
    } else {
        api::types::SearchMilestonesDetail::Slim
    };

    let mut req = client
        .search_milestones()
        .query(query)
        .page_size(args.page_size)
        .detail(detail);
    if let Some(next) = &args.next {
        req = req.next(next.clone());
    }

    let results = req
        .send()
        .await
        .map_err(|e| format!("Failed to search milestones: {e}"))?;

    for obj in &results.data {
        println!("{} - {} ({})", obj.id, obj.name, obj.state);
        if args.desc
            && let Some(d) = &obj.description
        {
            println!("  {d}");
        }
    }

    if results.data.is_empty() {
        println!("No milestones found");
    }

    print_pagination(results.data.len(), results.total, results.next.as_deref());

    Ok(())
}

async fn run_objectives(
    args: &SearchQueryArgs,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    let query = args
        .query
        .parse::<api::types::SearchObjectivesQuery>()
        .map_err(|e| format!("Invalid query: {e}"))?;

    let detail = if args.desc {
        api::types::SearchObjectivesDetail::Full
    } else {
        api::types::SearchObjectivesDetail::Slim
    };

    let mut req = client
        .search_objectives()
        .query(query)
        .page_size(args.page_size)
        .detail(detail);
    if let Some(next) = &args.next {
        req = req.next(next.clone());
    }

    let results = req
        .send()
        .await
        .map_err(|e| format!("Failed to search objectives: {e}"))?;

    for obj in &results.data {
        println!("{} - {} ({})", obj.id, obj.name, obj.state);
        if args.desc
            && let Some(d) = &obj.description
        {
            println!("  {d}");
        }
    }

    if results.data.is_empty() {
        println!("No objectives found");
    }

    print_pagination(results.data.len(), results.total, results.next.as_deref());

    Ok(())
}

async fn run_documents(args: &SearchQueryArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let title = args
        .query
        .parse::<api::types::SearchDocumentsTitle>()
        .map_err(|e| format!("Invalid query: {e}"))?;

    let mut req = client
        .search_documents()
        .title(title)
        .page_size(args.page_size);
    if let Some(next) = &args.next {
        req = req.next(next.clone());
    }

    let results = req
        .send()
        .await
        .map_err(|e| format!("Failed to search documents: {e}"))?;

    for doc in &results.data {
        let title = doc.title.as_deref().unwrap_or("(untitled)");
        println!("{} - {}", doc.id, title);
    }

    if results.data.is_empty() {
        println!("No documents found");
    }

    print_pagination(results.data.len(), results.total, results.next.as_deref());

    Ok(())
}

fn print_pagination(count: usize, total: i64, next: Option<&str>) {
    if count == 0 {
        return;
    }
    match next {
        Some(token) if !token.is_empty() => {
            println!("\nShowing {count} of {total} results. Use --next \"{token}\" for more.");
        }
        _ => {
            println!("\nShowing {count} of {total} results.");
        }
    }
}
