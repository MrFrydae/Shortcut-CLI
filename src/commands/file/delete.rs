use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(
    id: i64,
    confirm: bool,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if out.is_dry_run() {
        return out.dry_run_request::<serde_json::Value>(
            "DELETE",
            &format!("/api/v3/files/{id}"),
            None,
        );
    }

    if !confirm {
        return Err("Deleting a file is irreversible. Pass --confirm to proceed.".into());
    }

    let file = client
        .get_file()
        .file_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get file: {}", api::format_api_error(&e)))?;
    let name = file.name.clone();

    client
        .delete_file()
        .file_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete file: {}", api::format_api_error(&e)))?;

    if out.is_quiet() {
        return Ok(());
    }
    out_println!(out, "Deleted file {id} - {name}");
    Ok(())
}
