use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(id: i64, client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let project = client
        .get_project()
        .project_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get project: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*project)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        out_println!(out, "{}", project.id);
        return Ok(());
    }

    out_println!(out, "{} - {}", project.id, project.name);
    out_println!(
        out,
        "  Description: {}",
        project.description.as_deref().unwrap_or("none")
    );
    out_println!(
        out,
        "  Abbreviation: {}",
        project.abbreviation.as_deref().unwrap_or("none")
    );
    out_println!(
        out,
        "  Color:       {}",
        project.color.as_deref().unwrap_or("none")
    );
    out_println!(out, "  Team ID:     {}", project.team_id);
    out_println!(out, "  Workflow ID: {}", project.workflow_id);
    out_println!(out, "  Archived:    {}", project.archived);
    out_println!(
        out,
        "  Stats:       {} stories, {} points",
        project.stats.num_stories,
        project.stats.num_points
    );

    Ok(())
}
