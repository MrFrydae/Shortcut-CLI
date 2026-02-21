use std::error::Error;

use crate::api;

pub async fn run(id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let docs = client
        .list_epic_documents()
        .epic_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list epic documents: {e}"))?;

    for doc in docs.iter() {
        let title = doc.title.as_deref().unwrap_or("(untitled)");
        println!("{} - {}", doc.id, title);
    }

    if docs.is_empty() {
        println!("No documents linked to this epic");
    }

    Ok(())
}
