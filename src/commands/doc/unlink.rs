use std::error::Error;

use crate::api;

pub async fn run(doc_id: &str, epic_id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let doc_uuid: uuid::Uuid = doc_id
        .parse()
        .map_err(|e| format!("Invalid document UUID: {e}"))?;

    client
        .unlink_document_from_epic()
        .doc_public_id(doc_uuid)
        .epic_public_id(epic_id)
        .send()
        .await
        .map_err(|e| format!("Failed to unlink document from epic: {e}"))?;

    println!("Unlinked document {doc_id} from epic {epic_id}");
    Ok(())
}
