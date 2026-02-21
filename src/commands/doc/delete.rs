use std::error::Error;

use crate::api;

pub async fn run(id: &str, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a document is irreversible. Pass --confirm to proceed.".into());
    }

    let doc_id: uuid::Uuid = id
        .parse()
        .map_err(|e| format!("Invalid document UUID: {e}"))?;

    let doc = client
        .get_doc()
        .doc_public_id(doc_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get document: {e}"))?;

    let title = doc.title.as_deref().unwrap_or("(untitled)").to_string();

    client
        .delete_doc()
        .doc_public_id(doc_id)
        .body_map(|b| b)
        .send()
        .await
        .map_err(|e| format!("Failed to delete document: {e}"))?;

    println!("Deleted document {id} - {title}");
    Ok(())
}
