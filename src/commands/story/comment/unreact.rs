use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

fn format_emoji(emoji: &str) -> String {
    if emoji.starts_with(':') && emoji.ends_with(':') {
        emoji.to_string()
    } else {
        format!(":{emoji}:")
    }
}

pub async fn run(
    story_id: i64,
    comment_id: i64,
    emoji: &str,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let emoji_str = format_emoji(emoji);

    client
        .delete_story_reaction()
        .story_public_id(story_id)
        .comment_public_id(comment_id)
        .body_map(|b| b.emoji(emoji_str.clone()))
        .send()
        .await
        .map_err(|e| format!("Failed to remove reaction: {e}"))?;

    out_println!(
        out,
        "Removed {emoji_str} from comment #{comment_id} on story {story_id}"
    );
    Ok(())
}
