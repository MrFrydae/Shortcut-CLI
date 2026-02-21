use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(id: i64, client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let milestones = client
        .list_category_milestones()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list category milestones: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*milestones)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        for ms in milestones.iter() {
            out_println!(out, "{}", ms.id);
        }
        return Ok(());
    }

    for ms in milestones.iter() {
        out_println!(out, "{} - {} ({})", ms.id, ms.name, ms.state);
    }

    if milestones.is_empty() {
        out_println!(out, "No milestones for this category");
    }

    Ok(())
}
