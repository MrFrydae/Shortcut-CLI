use std::error::Error;

use crate::api;

pub async fn run(id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let iteration = client
        .get_iteration()
        .iteration_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get iteration: {e}"))?;

    println!("{} - {}", iteration.id, iteration.name);
    println!("  Status:      {}", iteration.status);
    println!("  Start:       {}", iteration.start_date);
    println!("  End:         {}", iteration.end_date);
    if !iteration.labels.is_empty() {
        let names: Vec<&str> = iteration.labels.iter().map(|l| l.name.as_str()).collect();
        println!("  Labels:      {}", names.join(", "));
    }

    let stats = &iteration.stats;
    let total_stories = stats.num_stories_started
        + stats.num_stories_done
        + stats.num_stories_unstarted
        + stats.num_stories_backlog;
    println!(
        "  Stories:     {} total, {} started, {} done, {} unstarted",
        total_stories,
        stats.num_stories_started,
        stats.num_stories_done,
        stats.num_stories_unstarted,
    );
    println!(
        "  Points:      {} total, {} started, {} done, {} unstarted",
        stats.num_points,
        stats.num_points_started,
        stats.num_points_done,
        stats.num_points_unstarted,
    );

    if let Some(cycle) = stats.average_cycle_time {
        let cycle_days = cycle as f64 / 86400.0;
        println!("  Avg Cycle:   {cycle_days:.1} days");
    }
    if let Some(lead) = stats.average_lead_time {
        let lead_days = lead as f64 / 86400.0;
        println!("  Avg Lead:    {lead_days:.1} days");
    }

    if !iteration.description.is_empty() {
        println!("  Description: {}", iteration.description);
    }

    Ok(())
}
