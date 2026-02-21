use std::error::Error;

use crate::api;

use super::SearchQueryArgs;
use super::helpers::print_pagination;

pub async fn run(args: &SearchQueryArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
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
