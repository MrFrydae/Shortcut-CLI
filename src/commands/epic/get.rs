use std::error::Error;
use std::path::Path;

use crate::api;
use crate::output::OutputConfig;

use super::helpers::{resolve_epic_state_name, resolve_member_name};
use crate::out_println;

pub async fn run(
    id: i64,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let epic = client
        .get_epic()
        .epic_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get epic: {e}"))?;

    if out.is_json() {
        let json = serde_json::json!({
            "id": epic.id,
            "name": epic.name,
            "epic_state_id": epic.epic_state_id,
            "description": epic.description,
        });
        out_println!(out, "{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }
    if out.is_quiet() {
        out_println!(out, "{}", epic.id);
        return Ok(());
    }

    let state_name = resolve_epic_state_name(epic.epic_state_id, client, cache_dir).await;

    out_println!(out, "{} - {}", epic.id, epic.name);
    out_println!(out, "  State:       {state_name}");
    if let Some(dl) = &epic.deadline {
        out_println!(out, "  Deadline:    {}", dl.format("%Y-%m-%d"));
    }
    let requested_by = resolve_member_name(&epic.requested_by_id, cache_dir);
    out_println!(out, "  Requested:   {requested_by}");
    if !epic.owner_ids.is_empty() {
        let owners: Vec<String> = epic
            .owner_ids
            .iter()
            .map(|id| resolve_member_name(id, cache_dir))
            .collect();
        out_println!(out, "  Owners:      {}", owners.join(", "));
    }
    if !epic.labels.is_empty() {
        let names: Vec<&str> = epic.labels.iter().map(|l| l.name.as_str()).collect();
        out_println!(out, "  Labels:      {}", names.join(", "));
    }
    if !epic.objective_ids.is_empty() {
        let ids: Vec<String> = epic.objective_ids.iter().map(|id| id.to_string()).collect();
        out_println!(out, "  Objectives:  {}", ids.join(", "));
    }
    out_println!(
        out,
        "  Stories:     {} total, {} started, {} done",
        epic.stats.num_stories_total,
        epic.stats.num_stories_started,
        epic.stats.num_stories_done,
    );
    if !epic.description.is_empty() {
        out_println!(out, "  Description: {}", epic.description);
    }

    Ok(())
}
