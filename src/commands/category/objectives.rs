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
    let objectives = client
        .list_category_objectives()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list category objectives: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*objectives)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        for obj in objectives.iter() {
            out_println!(out, "{}", obj.id);
        }
        return Ok(());
    }

    for obj in objectives.iter() {
        out_println!(out, "{} - {} ({})", obj.id, obj.name, obj.state);
        if desc {
            out_println!(out, "  {}", obj.description);
        }
    }

    if objectives.is_empty() {
        out_println!(out, "No objectives for this category");
    }

    Ok(())
}
