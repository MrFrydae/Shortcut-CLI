use std::error::Error;

use crate::api;

use super::SearchQueryArgs;

pub async fn run(args: &SearchQueryArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
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
