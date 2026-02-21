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
            &format!("/api/v3/iterations/{id}"),
            None,
        );
    }

    if !confirm {
        return Err("Deleting an iteration is irreversible. Pass --confirm to proceed.".into());
    }
    let iteration = client
        .get_iteration()
        .iteration_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get iteration: {e}"))?;
    let name = iteration.name.clone();
    client
        .delete_iteration()
        .iteration_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete iteration: {e}"))?;
    out_println!(out, "Deleted iteration {id} - {name}");
    Ok(())
}
