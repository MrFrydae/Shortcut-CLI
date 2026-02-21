use std::error::Error;
use std::path::Path;

use crate::api;
use crate::output::OutputConfig;

use super::helpers::resolve_member_name;
use crate::out_println;

pub async fn run(
    story_id: i64,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let comments = client
        .list_story_comment()
        .story_public_id(story_id)
        .send()
        .await
        .map_err(|e| format!("Failed to list comments: {e}"))?;

    if comments.is_empty() {
        out_println!(out, "No comments on story {story_id}");
        return Ok(());
    }

    out_println!(out, "Comments on story {story_id}:\n");
    for c in comments.iter() {
        let author = match &c.author_id {
            Some(uuid) => resolve_member_name(uuid, cache_dir),
            None => "unknown".to_string(),
        };
        let text_preview = c.text.as_deref().unwrap_or("").lines().next().unwrap_or("");
        out_println!(out, "  #{} {} ({})", c.id, author, c.created_at);
        out_println!(out, "    {text_preview}");
        out_println!(out, "");
    }
    Ok(())
}
