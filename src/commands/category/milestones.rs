use std::error::Error;

use crate::api;

pub async fn run(id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let milestones = client
        .list_category_milestones()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list category milestones: {e}"))?;

    for ms in milestones.iter() {
        println!("{} - {} ({})", ms.id, ms.name, ms.state);
    }

    if milestones.is_empty() {
        println!("No milestones for this category");
    }

    Ok(())
}
