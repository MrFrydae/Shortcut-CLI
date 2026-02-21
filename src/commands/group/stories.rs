use std::error::Error;
use std::path::Path;

use crate::api;
use crate::output::OutputConfig;

use super::helpers::resolve_group_id;
use crate::out_println;

pub async fn run(
    id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
    desc: bool,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
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
        out_println!(out, "No stories in this group");
    }

    Ok(())
}
