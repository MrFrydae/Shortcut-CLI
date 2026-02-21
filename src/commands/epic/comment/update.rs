use std::error::Error;

use crate::api;

pub async fn run(
    epic_id: i64,
    comment_id: i64,
    text: &str,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    client
        .update_epic_comment()
        .epic_public_id(epic_id)
        .comment_public_id(comment_id)
        .body_map(|b| b.text(text.to_string()))
        .send()
        .await
        .map_err(|e| format!("Failed to update comment: {e}"))?;

    println!("Updated comment #{comment_id} on epic {epic_id}");
    Ok(())
}
