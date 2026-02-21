use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(
    story_id: i64,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let story = client
        .get_story()
        .story_public_id(story_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get story: {e}"))?;

    if story.tasks.is_empty() {
        out_println!(out, "No tasks on story {story_id}");
        return Ok(());
    }

    for task in &story.tasks {
        let check = if task.complete { "x" } else { " " };
        out_println!(out, "[{check}] {} - {}", task.id, task.description);
    }
    Ok(())
}
