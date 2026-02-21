use std::error::Error;

use crate::api;

pub async fn run(client: &api::Client) -> Result<(), Box<dyn Error>> {
    let docs = client
        .list_docs()
        .send()
        .await
        .map_err(|e| format!("Failed to list documents: {e}"))?;

    for doc in docs.iter() {
        let title = doc.title.as_deref().unwrap_or("(untitled)");
        println!("{} - {}", doc.id, title);
    }

    if docs.is_empty() {
        println!("No documents found");
    }

    Ok(())
}
