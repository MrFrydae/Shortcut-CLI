use std::error::Error;

use crate::api;

pub async fn run(doc_id: &str, epic_id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let doc_uuid: uuid::Uuid = doc_id
        .parse()
        .map_err(|e| format!("Invalid document UUID: {e}"))?;

    client
        .link_document_to_epic()
        .doc_public_id(doc_uuid)
        .epic_public_id(epic_id)
        .send()
        .await
        .map_err(|e| format!("Failed to link document to epic: {e}"))?;

    println!("Linked document {doc_id} to epic {epic_id}");
    Ok(())
}
