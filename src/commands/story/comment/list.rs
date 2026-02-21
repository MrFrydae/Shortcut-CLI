use std::error::Error;
use std::path::Path;

use crate::api;

use super::helpers::resolve_member_name;

pub async fn run(
    story_id: i64,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let comments = client
        .list_story_comment()
        .story_public_id(story_id)
        .send()
        .await
        .map_err(|e| format!("Failed to list comments: {e}"))?;

    if comments.is_empty() {
        println!("No comments on story {story_id}");
        return Ok(());
    }

    println!("Comments on story {story_id}:\n");
    for c in comments.iter() {
        let author = match &c.author_id {
            Some(uuid) => resolve_member_name(uuid, cache_dir),
            None => "unknown".to_string(),
        };
        let text_preview = c.text.as_deref().unwrap_or("").lines().next().unwrap_or("");
        println!("  #{} {} ({})", c.id, author, c.created_at);
        println!("    {text_preview}");
        println!();
    }
    Ok(())
}
