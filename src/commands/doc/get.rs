use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(id: &str, client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let doc_id: uuid::Uuid = id
        .parse()
        .map_err(|e| format!("Invalid document UUID: {e}"))?;

    let doc = client
        .get_doc()
        .doc_public_id(doc_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get document: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*doc)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        out_println!(out, "{}", doc.id);
        return Ok(());
    }

    let title = doc.title.as_deref().unwrap_or("(untitled)");
    out_println!(out, "{} - {}", doc.id, title);
    out_println!(out, "  Archived:  {}", doc.archived);
    out_println!(out, "  Created:   {}", doc.created_at);
    out_println!(out, "  Updated:   {}", doc.updated_at);
    if let Some(content) = &doc.content_markdown {
        out_println!(out, "");
        out_println!(out, "{content}");
    }

    Ok(())
}
