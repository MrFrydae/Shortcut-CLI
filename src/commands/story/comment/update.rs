use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(
    story_id: i64,
    comment_id: i64,
    text: &str,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let text_value = text
        .parse::<api::types::UpdateStoryCommentText>()
        .map_err(|e| format!("Invalid comment text: {e}"))?;

    if out.is_dry_run() {
        let body = serde_json::json!({ "text": text });
        return out.dry_run_request(
            "PUT",
            &format!("/api/v3/stories/{story_id}/comments/{comment_id}"),
            Some(&body),
        );
    }

    client
        .update_story_comment()
        .story_public_id(story_id)
        .comment_public_id(comment_id)
        .body_map(|b| b.text(text_value))
        .send()
        .await
        .map_err(|e| format!("Failed to update comment: {e}"))?;

    out_println!(out, "Updated comment #{comment_id} on story {story_id}");
    Ok(())
}
