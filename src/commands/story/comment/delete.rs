use std::error::Error;

use crate::api;

pub async fn run(
    story_id: i64,
    comment_id: i64,
    confirm: bool,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a comment is irreversible. Pass --confirm to proceed.".into());
    }

    client
        .delete_story_comment()
        .story_public_id(story_id)
        .comment_public_id(comment_id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete comment: {e}"))?;

    println!("Deleted comment #{comment_id} from story {story_id}");
    Ok(())
}
