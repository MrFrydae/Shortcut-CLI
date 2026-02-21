use std::error::Error;
use std::path::Path;

use crate::api;
use crate::output::OutputConfig;

use super::helpers::print_threaded_comment;
use crate::out_println;

pub async fn run(
    epic_id: i64,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let comments = client
        .list_epic_comments()
        .epic_public_id(epic_id)
        .send()
        .await
        .map_err(|e| format!("Failed to list epic comments: {e}"))?;

    if comments.is_empty() {
        out_println!(out, "No comments on epic {epic_id}");
        return Ok(());
    }

    out_println!(out, "Comments on epic {epic_id}:\n");
    for comment in comments.iter() {
        print_threaded_comment(comment, cache_dir, 0, out)?;
    }
    Ok(())
}
