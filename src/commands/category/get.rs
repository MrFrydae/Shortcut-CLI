use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(id: i64, client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let category = client
        .get_category()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get category: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*category)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        out_println!(out, "{}", category.id);
        return Ok(());
    }

    out_println!(out, "{} - {}", category.id, category.name);
    out_println!(
        out,
        "  Color:       {}",
        category.color.as_deref().unwrap_or("none")
    );
    out_println!(out, "  Type:        {}", category.type_);
    out_println!(out, "  Archived:    {}", category.archived);

    // Show associated milestones
    let milestones = client
        .list_category_milestones()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list category milestones: {e}"))?;

    if !milestones.is_empty() {
        out_println!(out, "  Milestones:");
        for ms in milestones.iter() {
            out_println!(out, "    {} - {} ({})", ms.id, ms.name, ms.state);
        }
    }

    // Show associated objectives
    let objectives = client
        .list_category_objectives()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list category objectives: {e}"))?;

    if !objectives.is_empty() {
        out_println!(out, "  Objectives:");
        for obj in objectives.iter() {
            out_println!(out, "    {} - {} ({})", obj.id, obj.name, obj.state);
        }
    }

    Ok(())
}
