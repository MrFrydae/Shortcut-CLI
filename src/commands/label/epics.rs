use crate::api;
use crate::out_println;
use crate::output::OutputConfig;
use std::error::Error;

pub async fn run(
    id: i64,
    desc: bool,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let epics = client
        .list_label_epics()
        .label_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list label epics: {e}"))?;
    if epics.is_empty() {
        out_println!(out, "No epics with this label");
        return Ok(());
    }
    for epic in epics.iter() {
        out_println!(out, "{} - {}", epic.id, epic.name);
        if desc && let Some(d) = &epic.description {
            out_println!(out, "  {d}");
        }
    }
    Ok(())
}
