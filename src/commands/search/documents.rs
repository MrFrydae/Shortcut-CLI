use std::error::Error;

use crate::api;

use super::SearchQueryArgs;
use super::helpers::print_pagination;

pub async fn run(args: &SearchQueryArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
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
