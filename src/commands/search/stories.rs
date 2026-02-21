use std::error::Error;

use crate::api;

use super::SearchQueryArgs;
use super::helpers::print_pagination;

pub async fn run(args: &SearchQueryArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
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
