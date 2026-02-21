use std::error::Error;

use crate::api;

pub async fn run(include_archived: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let objectives = client
        .list_objectives()
        .send()
        .await
        .map_err(|e| format!("Failed to list objectives: {e}"))?;

    for obj in objectives.iter() {
        if !include_archived && obj.archived {
            continue;
        }
        println!("{} - {} ({})", obj.id, obj.name, obj.state);
    }
    Ok(())
}
