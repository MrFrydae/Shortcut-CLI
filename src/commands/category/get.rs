use std::error::Error;

use crate::api;

pub async fn run(id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let category = client
        .get_category()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get category: {e}"))?;

    println!("{} - {}", category.id, category.name);
    println!(
        "  Color:       {}",
        category.color.as_deref().unwrap_or("none")
    );
    println!("  Type:        {}", category.type_);
    println!("  Archived:    {}", category.archived);

    // Show associated milestones
    let milestones = client
        .list_category_milestones()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list category milestones: {e}"))?;

    if !milestones.is_empty() {
        println!("  Milestones:");
        for ms in milestones.iter() {
            println!("    {} - {} ({})", ms.id, ms.name, ms.state);
        }
    }

    // Show associated objectives
    let objectives = client
        .list_category_objectives()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list category objectives: {e}"))?;

    if !objectives.is_empty() {
        println!("  Objectives:");
        for obj in objectives.iter() {
            println!("    {} - {} ({})", obj.id, obj.name, obj.state);
        }
    }

    Ok(())
}
