use std::error::Error;

use crate::api;

pub async fn run(
    story_id: i64,
    comment_id: i64,
    text: &str,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    let text_value = text
        .parse::<api::types::UpdateStoryCommentText>()
        .map_err(|e| format!("Invalid comment text: {e}"))?;

    client
        .update_story_comment()
        .story_public_id(story_id)
        .comment_public_id(comment_id)
        .body_map(|b| b.text(text_value))
        .send()
        .await
        .map_err(|e| format!("Failed to update comment: {e}"))?;

    println!("Updated comment #{comment_id} on story {story_id}");
    Ok(())
}
