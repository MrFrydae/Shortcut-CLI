use std::error::Error;
use std::path::Path;

use crate::api;

use super::helpers::{print_threaded_comment, resolve_member_name};

pub async fn run(
    epic_id: i64,
    comment_id: i64,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let comment = client
        .get_epic_comment()
        .epic_public_id(epic_id)
        .comment_public_id(comment_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get comment: {e}"))?;

    let author = resolve_member_name(&comment.author_id, cache_dir);

    println!("Comment #{} on epic {epic_id}", comment.id);
    println!("  Author:  {author}");
    println!("  Created: {}", comment.created_at);
    println!("  Updated: {}", comment.updated_at);
    println!();
    println!("  {}", comment.text);
    if !comment.comments.is_empty() {
        println!();
        println!("  Replies:");
        for child in &comment.comments {
            print_threaded_comment(child, cache_dir, 2);
        }
    }
    Ok(())
}
