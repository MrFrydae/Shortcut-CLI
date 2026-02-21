use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(
    id: i64,
    desc: bool,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let epics = client
        .list_objective_epics()
        .objective_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list objective epics: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*epics)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        for epic in epics.iter() {
            out_println!(out, "{}", epic.id);
        }
        return Ok(());
    }

    for epic in epics.iter() {
        out_println!(out, "{} - {}", epic.id, epic.name);
        if desc && let Some(d) = &epic.description {
            out_println!(out, "  {d}");
        }
    }

    if epics.is_empty() {
        out_println!(out, "No epics for this objective");
    }

    Ok(())
}
