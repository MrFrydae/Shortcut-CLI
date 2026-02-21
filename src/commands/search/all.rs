use std::error::Error;

use crate::api;
use crate::output::OutputConfig;

use super::SearchQueryArgs;
use crate::out_println;

pub async fn run(
    args: &SearchQueryArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
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

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*results)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    let mut total = 0;

    if let Some(stories) = &results.stories {
        let count = stories.data.len();
        if count > 0 {
            let label = if count == 1 { "result" } else { "results" };
            out_println!(out, "Stories ({count} {label}):");
            for story in &stories.data {
                out_println!(
                    out,
                    "  {} - {} ({}, state_id: {})",
                    story.id,
                    story.name,
                    story.story_type,
                    story.workflow_state_id
                );
                if args.desc
                    && let Some(d) = &story.description
                {
                    out_println!(out, "    {d}");
                }
            }
            out_println!(out, "");
            total += count;
        }
    }

    if let Some(epics) = &results.epics {
        let count = epics.data.len();
        if count > 0 {
            let label = if count == 1 { "result" } else { "results" };
            out_println!(out, "Epics ({count} {label}):");
            for epic in &epics.data {
                out_println!(out, "  {} - {}", epic.id, epic.name);
                if args.desc
                    && let Some(d) = &epic.description
                {
                    out_println!(out, "    {d}");
                }
            }
            out_println!(out, "");
            total += count;
        }
    }

    if let Some(iterations) = &results.iterations {
        let count = iterations.data.len();
        if count > 0 {
            let label = if count == 1 { "result" } else { "results" };
            out_println!(out, "Iterations ({count} {label}):");
            for iter in &iterations.data {
                out_println!(
                    out,
                    "  {} - {} ({}, {} \u{2192} {})",
                    iter.id,
                    iter.name,
                    iter.status,
                    iter.start_date,
                    iter.end_date
                );
            }
            out_println!(out, "");
            total += count;
        }
    }

    if let Some(milestones) = &results.milestones {
        let count = milestones.data.len();
        if count > 0 {
            let label = if count == 1 { "result" } else { "results" };
            out_println!(out, "Objectives ({count} {label}):");
            for obj in &milestones.data {
                out_println!(out, "  {} - {} ({})", obj.id, obj.name, obj.state);
                if args.desc
                    && let Some(d) = &obj.description
                {
                    out_println!(out, "    {d}");
                }
            }
            out_println!(out, "");
            total += count;
        }
    }

    if total == 0 {
        out_println!(out, "No results found");
    } else {
        let label = if total == 1 { "result" } else { "results" };
        out_println!(out, "Showing {total} {label}.");
    }

    Ok(())
}
