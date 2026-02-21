use std::error::Error;

use crate::api;

pub async fn run(story_id: i64, task_id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let task = client
        .get_task()
        .story_public_id(story_id)
        .task_public_id(task_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get task: {e}"))?;

    let check = if task.complete { "x" } else { " " };
    println!("[{check}] {} - {}", task.id, task.description);
    Ok(())
}
