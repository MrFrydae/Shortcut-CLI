use std::error::Error;

use crate::api;

pub async fn run(id: i64, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
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

    println!("Deleted objective {id} - {name}");
    Ok(())
}
