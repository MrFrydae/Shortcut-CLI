use std::error::Error;

use crate::api;

use super::SearchQueryArgs;
use super::helpers::print_pagination;

pub async fn run(args: &SearchQueryArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
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
