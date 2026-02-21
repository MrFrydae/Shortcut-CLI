use std::error::Error;

use clap::Args;

use crate::api;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateTaskArgs {
    /// The story ID
    #[arg(long)]
    pub story_id: i64,
    /// The task ID
    #[arg(long)]
    pub id: i64,
    /// New description for the task
    #[arg(long)]
    pub description: Option<String>,
    /// Mark the task as complete or incomplete
    #[arg(long)]
    pub complete: Option<bool>,
}

pub async fn run(args: &UpdateTaskArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateTaskDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let task = client
        .update_task()
        .story_public_id(args.story_id)
        .task_public_id(args.id)
        .body_map(|mut b| {
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(complete) = args.complete {
                b = b.complete(Some(complete));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update task: {e}"))?;

    println!("Updated task {} - {}", task.id, task.description);
    Ok(())
}
