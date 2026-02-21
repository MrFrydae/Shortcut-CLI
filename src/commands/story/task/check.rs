use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(
    story_id: i64,
    task_id: i64,
    complete: bool,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if out.is_dry_run() {
        let body = serde_json::json!({ "complete": complete });
        return out.dry_run_request(
            "PUT",
            &format!("/api/v3/stories/{story_id}/tasks/{task_id}"),
            Some(&body),
        );
    }

    let task = client
        .update_task()
        .story_public_id(story_id)
        .task_public_id(task_id)
        .body_map(|b| b.complete(Some(complete)))
        .send()
        .await
        .map_err(|e| format!("Failed to update task: {e}"))?;

    let status = if task.complete {
        "complete"
    } else {
        "incomplete"
    };
    out_println!(out, "Task {} marked {status}", task.id);
    Ok(())
}
