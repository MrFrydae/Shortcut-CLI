use crate::api;
use crate::out_println;
use crate::output::OutputConfig;
use std::error::Error;

pub async fn run(id: i64, client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let label = client
        .get_label()
        .label_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get label: {e}"))?;
    if out.is_quiet() {
        out_println!(out, "{}", label.id);
        return Ok(());
    }
    let color = label
        .color
        .as_deref()
        .map(|c| format!(" ({c})"))
        .unwrap_or_default();
    out_println!(out, "{} - {}{}", label.id, label.name, color);
    if let Some(d) = &label.description
        && !d.is_empty()
    {
        out_println!(out, "  Description: {d}");
    }
    out_println!(out, "  Archived:    {}", label.archived);
    if let Some(stats) = &label.stats {
        out_println!(out, "  Stories:     {}", stats.num_stories_total);
        out_println!(out, "  Epics:       {}", stats.num_epics_total);
    }
    Ok(())
}
