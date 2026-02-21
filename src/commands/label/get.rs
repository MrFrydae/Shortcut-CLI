use std::error::Error;

use crate::api;

pub async fn run(id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let label = client
        .get_label()
        .label_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get label: {e}"))?;

    let color = label
        .color
        .as_deref()
        .map(|c| format!(" ({c})"))
        .unwrap_or_default();
    println!("{} - {}{}", label.id, label.name, color);
    if let Some(d) = &label.description {
        println!("  Description: {d}");
    }
    println!("  Archived:    {}", label.archived);
    if let Some(stats) = &label.stats {
        println!("  Stories:     {}", stats.num_stories_total);
        println!("  Epics:       {}", stats.num_epics_total);
    }

    Ok(())
}
