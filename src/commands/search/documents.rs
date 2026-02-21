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

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*results)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        for doc in &results.data {
            out_println!(out, "{}", doc.id);
        }
        return Ok(());
    }

    for doc in &results.data {
        let title = doc.title.as_deref().unwrap_or("(untitled)");
        out_println!(out, "{} - {}", doc.id, title);
    }

    if results.data.is_empty() {
        out_println!(out, "No documents found");
    }

    print_pagination(
        results.data.len(),
        results.total,
        results.next.as_deref(),
        out,
    );

    Ok(())
}
