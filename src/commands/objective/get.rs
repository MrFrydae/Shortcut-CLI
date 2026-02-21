use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(id: i64, client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let objective = client
        .get_objective()
        .objective_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get objective: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*objective)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        out_println!(out, "{}", objective.id);
        return Ok(());
    }

    out_println!(out, "{} - {}", objective.id, objective.name);
    out_println!(out, "  State:       {}", objective.state);
    out_println!(out, "  Archived:    {}", objective.archived);
    out_println!(out, "  Started:     {}", objective.started);
    out_println!(out, "  Completed:   {}", objective.completed);

    if !objective.categories.is_empty() {
        let names: Vec<&str> = objective
            .categories
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        out_println!(out, "  Categories:  {}", names.join(", "));
    }

    let stats = &objective.stats;
    out_println!(out, "  Documents:   {}", stats.num_related_documents);
    if let Some(cycle) = stats.average_cycle_time {
        let cycle_days = cycle as f64 / 86400.0;
        out_println!(out, "  Avg Cycle:   {cycle_days:.1} days");
    }
    if let Some(lead) = stats.average_lead_time {
        let lead_days = lead as f64 / 86400.0;
        out_println!(out, "  Avg Lead:    {lead_days:.1} days");
    }

    if !objective.description.is_empty() {
        out_println!(out, "  Description: {}", objective.description);
    }

    // Show associated epics
    let epics = client
        .list_objective_epics()
        .objective_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list objective epics: {e}"))?;

    if !epics.is_empty() {
        out_println!(out, "  Epics:");
        for epic in epics.iter() {
            out_println!(out, "    {} - {}", epic.id, epic.name);
        }
    }

    Ok(())
}
