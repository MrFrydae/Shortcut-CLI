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
    let mut req = client.list_stories().project_public_id(id);
    if desc {
        req = req.includes_description(true);
    }
    let stories = req
        .send()
        .await
        .map_err(|e| format!("Failed to list project stories: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*stories)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        for story in stories.iter() {
            out_println!(out, "{}", story.id);
        }
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

    if stories.is_empty() {
        out_println!(out, "No stories in this project");
    }

    Ok(())
}
