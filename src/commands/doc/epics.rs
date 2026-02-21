use std::error::Error;

use crate::api;

pub async fn run(id: &str, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let doc_id: uuid::Uuid = id
        .parse()
        .map_err(|e| format!("Invalid document UUID: {e}"))?;

    let epics = client
        .list_document_epics()
        .doc_public_id(doc_id)
        .send()
        .await
        .map_err(|e| format!("Failed to list document epics: {e}"))?;

    for epic in epics.iter() {
        println!("{} - {}", epic.id, epic.name);
    }

    if epics.is_empty() {
        println!("No epics linked to this document");
    }

    Ok(())
}
