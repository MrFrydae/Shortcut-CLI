use std::error::Error;
use std::path::Path;

use crate::api;

use super::helpers::resolve_group_id;

pub async fn run(
    id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
    desc: bool,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let group_id = resolve_group_id(id, client, cache_dir).await?;

    let mut req = client.list_group_stories().group_public_id(group_id);
    if let Some(limit) = limit {
        req = req.limit(limit);
    }
    if let Some(offset) = offset {
        req = req.offset(offset);
    }
    let stories = req
        .send()
        .await
        .map_err(|e| format!("Failed to list group stories: {e}"))?;

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
        println!("No stories in this group");
    }

    Ok(())
}
