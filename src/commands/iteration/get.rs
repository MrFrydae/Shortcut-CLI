use crate::api;
use crate::out_println;
use crate::output::OutputConfig;
use std::error::Error;

pub async fn run(id: i64, client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let iteration = client
        .get_iteration()
        .iteration_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get iteration: {e}"))?;

    if out.is_quiet() {
        out_println!(out, "{}", iteration.id);
        return Ok(());
    }

    out_println!(out, "{} - {}", iteration.id, iteration.name);
    out_println!(out, "  Status:      {}", iteration.status);
    out_println!(out, "  Start:       {}", iteration.start_date);
    out_println!(out, "  End:         {}", iteration.end_date);
    if !iteration.labels.is_empty() {
        let names: Vec<&str> = iteration.labels.iter().map(|l| l.name.as_str()).collect();
        out_println!(out, "  Labels:      {}", names.join(", "));
    }
    let stats = &iteration.stats;
    let total_stories =
        stats.num_stories_started + stats.num_stories_done + stats.num_stories_unstarted;
    out_println!(
        out,
        "  Stories:     {} total, {} started, {} done, {} unstarted",
        total_stories,
        stats.num_stories_started,
        stats.num_stories_done,
        stats.num_stories_unstarted
    );
    out_println!(
        out,
        "  Points:      {} total, {} started, {} done, {} unstarted",
        stats.num_points,
        stats.num_points_started,
        stats.num_points_done,
        stats.num_points_unstarted
    );
    if let Some(cycle) = &iteration.stats.average_cycle_time {
        let cycle_days = *cycle as f64 / 86400.0;
        out_println!(out, "  Avg Cycle:   {cycle_days:.1} days");
    }
    if let Some(lead) = &iteration.stats.average_lead_time {
        let lead_days = *lead as f64 / 86400.0;
        out_println!(out, "  Avg Lead:    {lead_days:.1} days");
    }
    if !iteration.description.is_empty() {
        out_println!(out, "  Description: {}", iteration.description);
    }
    Ok(())
}
