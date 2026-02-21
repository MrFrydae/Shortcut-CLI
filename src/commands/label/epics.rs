use std::error::Error;

use crate::api;

pub async fn run(id: i64, desc: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let epics = client
        .list_label_epics()
        .label_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list label epics: {e}"))?;

    for epic in epics.iter() {
        println!("{} - {}", epic.id, epic.name);
        if desc && let Some(d) = &epic.description {
            println!("  {d}");
        }
    }

    if epics.is_empty() {
        println!("No epics with this label");
    }

    Ok(())
}
