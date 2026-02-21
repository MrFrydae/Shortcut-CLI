use std::error::Error;
use std::path::Path;

use crate::api;

use super::helpers::resolve_member_name;

pub async fn run(
    story_id: i64,
    comment_id: i64,
    client: &api::Client,
    cache_dir: &Path,
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

    println!("Comment #{} on story {story_id}", comment.id);
    println!("  Author:  {author}");
    println!("  Created: {}", comment.created_at);
    if let Some(updated) = &comment.updated_at {
        println!("  Updated: {updated}");
    }
    println!();
    if let Some(text) = &comment.text {
        println!("  {text}");
    }
    if !comment.reactions.is_empty() {
        println!();
        let reactions: Vec<String> = comment
            .reactions
            .iter()
            .map(|r| {
                let name = r.emoji.trim_matches(':');
                format!("{name} ({})", r.permission_ids.len())
            })
            .collect();
        println!("  Reactions: {}", reactions.join(", "));
    }
    Ok(())
}
