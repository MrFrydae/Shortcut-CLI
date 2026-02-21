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
        return Err("Deleting an epic is irreversible. Pass --confirm to proceed.".into());
    }

    let epic = client
        .get_epic()
        .epic_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get epic: {e}"))?;

    let name = epic.name.clone();

    client
        .delete_epic()
        .epic_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete epic: {e}"))?;

    out_println!(out, "Deleted epic {id} - {name}");
    Ok(())
}
