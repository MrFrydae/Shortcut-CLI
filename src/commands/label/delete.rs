use crate::api;
use crate::out_println;
use crate::output::OutputConfig;
use std::error::Error;

pub async fn run(
    id: i64,
    confirm: bool,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if out.is_dry_run() {
        return out.dry_run_request::<serde_json::Value>(
            "DELETE",
            &format!("/api/v3/labels/{id}"),
            None,
        );
    }

    if !confirm {
        return Err("Deleting a label is irreversible. Pass --confirm to proceed.".into());
    }
    let label = client
        .get_label()
        .label_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get label: {}", crate::api::format_api_error(&e)))?;
    let name = label.name.clone();
    client
        .delete_label()
        .label_public_id(id)
        .send()
        .await
        .map_err(|e| {
            format!(
                "Failed to delete label: {}",
                crate::api::format_api_error(&e)
            )
        })?;
    out_println!(out, "Deleted label {id} - {name}");
    Ok(())
}
