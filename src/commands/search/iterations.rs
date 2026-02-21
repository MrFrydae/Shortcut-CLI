use std::error::Error;

use crate::api;

use super::SearchQueryArgs;
use super::helpers::print_pagination;

pub async fn run(args: &SearchQueryArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
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
