use std::error::Error;

use crate::api;
use crate::output::OutputConfig;

use super::SearchQueryArgs;
use super::helpers::print_pagination;
use crate::out_println;

pub async fn run(
    args: &SearchQueryArgs,
    client: &api::Client,
    out: &OutputConfig,
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

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*results)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        for obj in &results.data {
            out_println!(out, "{}", obj.id);
        }
        return Ok(());
    }

    for obj in &results.data {
        out_println!(out, "{} - {} ({})", obj.id, obj.name, obj.state);
        if args.desc
            && let Some(d) = &obj.description
        {
            out_println!(out, "  {d}");
        }
    }

    if results.data.is_empty() {
        out_println!(out, "No milestones found");
    }

    print_pagination(
        results.data.len(),
        results.total,
        results.next.as_deref(),
        out,
    );

    Ok(())
}
