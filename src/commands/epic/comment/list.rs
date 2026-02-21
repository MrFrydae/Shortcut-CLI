use std::error::Error;
use std::path::Path;

use crate::api;

use super::helpers::print_threaded_comment;

pub async fn run(
    epic_id: i64,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let comments = client
        .list_epic_comments()
        .epic_public_id(epic_id)
        .send()
        .await
        .map_err(|e| format!("Failed to list comments: {e}"))?;

    if comments.is_empty() {
        println!("No comments on epic {epic_id}");
        return Ok(());
    }

    println!("Comments on epic {epic_id}:\n");
    for c in comments.iter() {
        print_threaded_comment(c, cache_dir, 1);
    }
    Ok(())
}
