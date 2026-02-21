use std::error::Error;

use crate::api;

pub async fn run(id: i64, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
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

    println!("Deleted iteration {id} - {name}");
    Ok(())
}
