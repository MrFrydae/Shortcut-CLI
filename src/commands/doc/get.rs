use std::error::Error;

use crate::api;

pub async fn run(id: &str, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let doc_id: uuid::Uuid = id
        .parse()
        .map_err(|e| format!("Invalid document UUID: {e}"))?;

    let doc = client
        .get_doc()
        .doc_public_id(doc_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get document: {e}"))?;

    let title = doc.title.as_deref().unwrap_or("(untitled)");
    println!("{} - {}", doc.id, title);
    println!("  Archived:  {}", doc.archived);
    println!("  Created:   {}", doc.created_at);
    println!("  Updated:   {}", doc.updated_at);
    if let Some(content) = &doc.content_markdown {
        println!();
        println!("{content}");
    }

    Ok(())
}
