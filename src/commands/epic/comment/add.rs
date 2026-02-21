use std::error::Error;
use std::path::Path;

use crate::api;

pub async fn run(
    epic_id: i64,
    text: Option<&str>,
    text_file: Option<&Path>,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    let body = if let Some(path) = text_file {
        std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {e}", path.display()))?
    } else if let Some(t) = text {
        t.to_string()
    } else {
        return Err("Either --text or --text-file is required".into());
    };

    let text_value = body
        .parse::<api::types::CreateEpicCommentText>()
        .map_err(|e| format!("Invalid comment text: {e}"))?;

    let comment = client
        .create_epic_comment()
        .epic_public_id(epic_id)
        .body_map(|b| b.text(text_value))
        .send()
        .await
        .map_err(|e| format!("Failed to create comment: {e}"))?;

    println!("Created comment #{} on epic {epic_id}", comment.id);
    Ok(())
}
