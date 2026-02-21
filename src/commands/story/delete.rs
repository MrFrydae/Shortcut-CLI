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
            &format!("/api/v3/stories/{id}"),
            None,
        );
    }

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

    if out.is_quiet() {
        out_println!(out, "{id}");
        return Ok(());
    }
    out_println!(out, "Deleted story {id} - {name}");
    Ok(())
}
