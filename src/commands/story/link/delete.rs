use std::error::Error;

use crate::api;

pub async fn run(id: i64, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a story link is irreversible. Pass --confirm to proceed.".into());
    }

    let link = client
        .get_story_link()
        .story_link_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get story link: {e}"))?;

    client
        .delete_story_link()
        .story_link_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete story link: {e}"))?;

    println!(
        "Deleted story link {} ({} {} {})",
        link.id, link.subject_id, link.verb, link.object_id
    );
    Ok(())
}
