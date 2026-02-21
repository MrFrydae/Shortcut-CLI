use std::error::Error;

use crate::api;

pub async fn run(id: i64, desc: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let mut req = client.list_label_stories().label_public_id(id);
    if desc {
        req = req.includes_description(true);
    }
    let stories = req
        .send()
        .await
        .map_err(|e| format!("Failed to list label stories: {e}"))?;

    for story in stories.iter() {
        println!(
            "{} - {} ({}, state_id: {})",
            story.id, story.name, story.story_type, story.workflow_state_id
        );
        if desc && let Some(d) = &story.description {
            println!("  {d}");
        }
    }

    if stories.is_empty() {
        println!("No stories with this label");
    }

    Ok(())
}
