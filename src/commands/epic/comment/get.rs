use std::error::Error;
use std::path::Path;

use crate::api;
use crate::output::OutputConfig;

use super::helpers::{print_threaded_comment, resolve_member_name};
use crate::out_println;

pub async fn run(
    epic_id: i64,
    comment_id: i64,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let comment = client
        .get_epic_comment()
        .epic_public_id(epic_id)
        .comment_public_id(comment_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get comment: {e}"))?;

    let author = resolve_member_name(&comment.author_id, cache_dir);

    out_println!(out, "Comment #{} on epic {epic_id}", comment.id);
    out_println!(out, "  Author:  {author}");
    out_println!(out, "  Created: {}", comment.created_at);
    out_println!(out, "  Updated: {}", comment.updated_at);
    out_println!(out, "");
    out_println!(out, "  {}", comment.text);

    if !comment.comments.is_empty() {
        out_println!(out, "");
        out_println!(out, "  Replies:");
        for reply in &comment.comments {
            print_threaded_comment(reply, cache_dir, 2, out)?;
        }
    }
    Ok(())
}
