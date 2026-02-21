use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(
    doc_id: &str,
    epic_id: i64,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if out.is_dry_run() {
        return out.dry_run_request::<serde_json::Value>(
            "DELETE",
            &format!("/api/v3/docs/{doc_id}/epics/{epic_id}"),
            None,
        );
    }

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

    if out.is_quiet() {
        return Ok(());
    }

    out_println!(out, "Unlinked document {doc_id} from epic {epic_id}");
    Ok(())
}
