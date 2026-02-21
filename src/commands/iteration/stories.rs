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
    let stories = client
        .list_iteration_stories()
        .iteration_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list stories: {e}"))?;
    if stories.is_empty() {
        out_println!(out, "No stories in this iteration");
        return Ok(());
    }
    for story in stories.iter() {
        out_println!(
            out,
            "{} - {} ({}, state_id: {})",
            story.id,
            story.name,
            story.story_type,
            story.workflow_state_id
        );
        if desc && let Some(d) = &story.description {
            out_println!(out, "  {d}");
        }
    }
    Ok(())
}
