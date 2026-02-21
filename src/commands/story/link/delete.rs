use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(
    id: i64,
    confirm: bool,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if out.is_dry_run() {
        return out.dry_run_request::<serde_json::Value>(
            "DELETE",
            &format!("/api/v3/story-links/{id}"),
            None,
        );
    }

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

    out_println!(
        out,
        "Deleted story link {} ({} {} {})",
        link.id,
        link.subject_id,
        link.verb,
        link.object_id
    );
    Ok(())
}
