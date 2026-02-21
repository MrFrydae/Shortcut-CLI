use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(
    id: &str,
    confirm: bool,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a template is irreversible. Pass --confirm to proceed.".into());
    }

    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e| format!("Invalid entity template UUID: {e}"))?;

    let template = client
        .get_entity_template()
        .entity_template_public_id(uuid)
        .send()
        .await
        .map_err(|e| format!("Failed to get entity template: {e}"))?;

    let name = template.name.clone();

    client
        .delete_entity_template()
        .entity_template_public_id(uuid)
        .send()
        .await
        .map_err(|e| format!("Failed to delete entity template: {e}"))?;

    if out.is_quiet() {
        return Ok(());
    }

    out_println!(out, "Deleted template {id} - {name}");
    Ok(())
}
