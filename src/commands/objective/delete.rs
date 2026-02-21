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
    if !confirm {
        return Err("Deleting an objective is irreversible. Pass --confirm to proceed.".into());
    }

    let objective = client
        .get_objective()
        .objective_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get objective: {e}"))?;

    let name = objective.name.clone();

    client
        .delete_objective()
        .objective_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete objective: {e}"))?;

    if out.is_quiet() {
        return Ok(());
    }

    out_println!(out, "Deleted objective {id} - {name}");
    Ok(())
}
