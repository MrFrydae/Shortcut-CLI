use std::error::Error;

use crate::api;

pub async fn run(id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let objective = client
        .get_objective()
        .objective_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get objective: {e}"))?;

    println!("{} - {}", objective.id, objective.name);
    println!("  State:       {}", objective.state);
    println!("  Archived:    {}", objective.archived);
    println!("  Started:     {}", objective.started);
    println!("  Completed:   {}", objective.completed);

    if !objective.categories.is_empty() {
        let names: Vec<&str> = objective
            .categories
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        println!("  Categories:  {}", names.join(", "));
    }

    let stats = &objective.stats;
    println!("  Documents:   {}", stats.num_related_documents);
    if let Some(cycle) = stats.average_cycle_time {
        let cycle_days = cycle as f64 / 86400.0;
        println!("  Avg Cycle:   {cycle_days:.1} days");
    }
    if let Some(lead) = stats.average_lead_time {
        let lead_days = lead as f64 / 86400.0;
        println!("  Avg Lead:    {lead_days:.1} days");
    }

    if !objective.description.is_empty() {
        println!("  Description: {}", objective.description);
    }

    // Show associated epics
    let epics = client
        .list_objective_epics()
        .objective_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list objective epics: {e}"))?;

    if !epics.is_empty() {
        println!("  Epics:");
        for epic in epics.iter() {
            println!("    {} - {}", epic.id, epic.name);
        }
    }

    Ok(())
}
