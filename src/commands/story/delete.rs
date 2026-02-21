use std::error::Error;

use crate::api;

pub async fn run(id: i64, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a story is irreversible. Pass --confirm to proceed.".into());
    }

    let story = client
        .get_story()
        .story_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get story: {e}"))?;

    let name = story.name.clone();

    client
        .delete_story()
        .story_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete story: {e}"))?;

    println!("Deleted story {id} - {name}");
    Ok(())
}
