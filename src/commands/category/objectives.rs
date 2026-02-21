use std::error::Error;

use crate::api;

pub async fn run(id: i64, desc: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let objectives = client
        .list_category_objectives()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list category objectives: {e}"))?;

    for obj in objectives.iter() {
        println!("{} - {} ({})", obj.id, obj.name, obj.state);
        if desc {
            println!("  {}", obj.description);
        }
    }

    if objectives.is_empty() {
        println!("No objectives for this category");
    }

    Ok(())
}
