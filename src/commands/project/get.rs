use std::error::Error;

use crate::api;

pub async fn run(id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let project = client
        .get_project()
        .project_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get project: {e}"))?;

    println!("{} - {}", project.id, project.name);
    println!(
        "  Description: {}",
        project.description.as_deref().unwrap_or("none")
    );
    println!(
        "  Abbreviation: {}",
        project.abbreviation.as_deref().unwrap_or("none")
    );
    println!(
        "  Color:       {}",
        project.color.as_deref().unwrap_or("none")
    );
    println!("  Team ID:     {}", project.team_id);
    println!("  Workflow ID: {}", project.workflow_id);
    println!("  Archived:    {}", project.archived);
    println!(
        "  Stats:       {} stories, {} points",
        project.stats.num_stories, project.stats.num_points
    );

    Ok(())
}
