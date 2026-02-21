use std::error::Error;
use std::path::Path;

use crate::api;

use super::helpers::{resolve_epic_state_name, resolve_member_name};

pub async fn run(id: i64, client: &api::Client, cache_dir: &Path) -> Result<(), Box<dyn Error>> {
    let epic = client
        .get_epic()
        .epic_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get epic: {e}"))?;

    let state_name = resolve_epic_state_name(epic.epic_state_id, client, cache_dir).await;

    println!("{} - {}", epic.id, epic.name);
    println!("  State:       {state_name}");
    if let Some(dl) = &epic.deadline {
        println!("  Deadline:    {}", dl.format("%Y-%m-%d"));
    }
    let requested_by = resolve_member_name(&epic.requested_by_id, cache_dir);
    println!("  Requested:   {requested_by}");
    if !epic.owner_ids.is_empty() {
        let owners: Vec<String> = epic
            .owner_ids
            .iter()
            .map(|id| resolve_member_name(id, cache_dir))
            .collect();
        println!("  Owners:      {}", owners.join(", "));
    }
    if !epic.labels.is_empty() {
        let names: Vec<&str> = epic.labels.iter().map(|l| l.name.as_str()).collect();
        println!("  Labels:      {}", names.join(", "));
    }
    if !epic.objective_ids.is_empty() {
        let ids: Vec<String> = epic.objective_ids.iter().map(|id| id.to_string()).collect();
        println!("  Objectives:  {}", ids.join(", "));
    }
    println!(
        "  Stories:     {} total, {} started, {} done",
        epic.stats.num_stories_total, epic.stats.num_stories_started, epic.stats.num_stories_done,
    );
    if !epic.description.is_empty() {
        println!("  Description: {}", epic.description);
    }

    Ok(())
}
