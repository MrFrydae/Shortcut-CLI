use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(id: i64, client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let docs = client
        .list_epic_documents()
        .epic_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list related documents: {e}"))?;

    if docs.is_empty() {
        out_println!(out, "No documents linked to this epic");
        return Ok(());
    }

    for doc in docs.iter() {
        let title = doc.title.as_deref().unwrap_or("(untitled)");
        out_println!(out, "{} - {}", doc.id, title);
    }
    Ok(())
}
