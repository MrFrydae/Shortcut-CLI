use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(
    story_id: i64,
    task_id: i64,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if out.is_dry_run() {
        return out.dry_run_request::<serde_json::Value>(
            "DELETE",
            &format!("/api/v3/stories/{story_id}/tasks/{task_id}"),
            None,
        );
    }

    client
        .delete_task()
        .story_public_id(story_id)
        .task_public_id(task_id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete task: {e}"))?;

    out_println!(out, "Deleted task {task_id} from story {story_id}");
    Ok(())
}
