use std::error::Error;
use std::path::Path;

use crate::api;
use crate::output::OutputConfig;

use super::helpers::resolve_member_name;
use crate::out_println;

pub async fn run(
    story_id: i64,
    comment_id: i64,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let comment = client
        .get_story_comment()
        .story_public_id(story_id)
        .comment_public_id(comment_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get comment: {e}"))?;

    let author = match &comment.author_id {
        Some(uuid) => resolve_member_name(uuid, cache_dir),
        None => "unknown".to_string(),
    };

    out_println!(out, "Comment #{} on story {story_id}", comment.id);
    out_println!(out, "  Author:  {author}");
    out_println!(out, "  Created: {}", comment.created_at);
    if let Some(updated) = &comment.updated_at {
        out_println!(out, "  Updated: {updated}");
    }
    out_println!(out, "");
    if let Some(text) = &comment.text {
        out_println!(out, "  {text}");
    }
    if !comment.reactions.is_empty() {
        out_println!(out, "");
        let reactions: Vec<String> = comment
            .reactions
            .iter()
            .map(|r| {
                let name = r.emoji.trim_matches(':');
                format!("{name} ({})", r.permission_ids.len())
            })
            .collect();
        out_println!(out, "  Reactions: {}", reactions.join(", "));
    }
    Ok(())
}
