use std::error::Error;

use crate::api;

pub async fn run(include_archived: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let categories = client
        .list_categories()
        .send()
        .await
        .map_err(|e| format!("Failed to list categories: {e}"))?;

    for cat in categories.iter() {
        if !include_archived && cat.archived {
            continue;
        }
        let color = cat.color.as_deref().unwrap_or("none");
        println!("{} - {} ({})", cat.id, cat.name, color);
    }
    Ok(())
}
