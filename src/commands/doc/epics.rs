use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(id: &str, client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let doc_id: uuid::Uuid = id
        .parse()
        .map_err(|e| format!("Invalid document UUID: {e}"))?;

    let epics = client
        .list_document_epics()
        .doc_public_id(doc_id)
        .send()
        .await
        .map_err(|e| format!("Failed to list document epics: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*epics)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        for epic in epics.iter() {
            out_println!(out, "{}", epic.id);
        }
        return Ok(());
    }

    for epic in epics.iter() {
        out_println!(out, "{} - {}", epic.id, epic.name);
    }

    if epics.is_empty() {
        out_println!(out, "No epics linked to this document");
    }

    Ok(())
}
