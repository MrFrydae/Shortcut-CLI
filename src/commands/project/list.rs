use std::error::Error;

use crate::api;

pub async fn run(include_archived: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let projects = client
        .list_projects()
        .send()
        .await
        .map_err(|e| format!("Failed to list projects: {e}"))?;

    for proj in projects.iter() {
        if !include_archived && proj.archived {
            continue;
        }
        println!(
            "{} - {} ({} stories)",
            proj.id, proj.name, proj.stats.num_stories
        );
    }
    Ok(())
}
