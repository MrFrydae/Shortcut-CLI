use std::error::Error;

use crate::api;

pub async fn run(
    story_id: i64,
    task_id: i64,
    complete: bool,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
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
    println!("Task {} marked {status}", task.id);
    Ok(())
}
