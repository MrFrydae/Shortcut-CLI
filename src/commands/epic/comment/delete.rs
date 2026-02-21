use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(
    epic_id: i64,
    comment_id: i64,
    confirm: bool,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if out.is_dry_run() {
        return out.dry_run_request::<serde_json::Value>(
            "DELETE",
            &format!("/api/v3/epics/{epic_id}/comments/{comment_id}"),
            None,
        );
    }

    if !confirm {
        return Err("Deleting a comment is irreversible. Pass --confirm to proceed.".into());
    }

    client
        .delete_epic_comment()
        .epic_public_id(epic_id)
        .comment_public_id(comment_id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete comment: {e}"))?;

    out_println!(out, "Deleted comment #{comment_id} from epic {epic_id}");
    Ok(())
}
