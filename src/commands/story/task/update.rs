use std::error::Error;

use clap::Args;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

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

pub async fn run(
    args: &UpdateTaskArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateTaskDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    if out.is_dry_run() {
        let mut body = serde_json::Map::new();
        if let Some(desc) = &args.description {
            body.insert("description".into(), serde_json::json!(desc));
        }
        if let Some(complete) = args.complete {
            body.insert("complete".into(), serde_json::json!(complete));
        }
        let body = serde_json::Value::Object(body);
        return out.dry_run_request(
            "PUT",
            &format!("/api/v3/stories/{}/tasks/{}", args.story_id, args.id),
            Some(&body),
        );
    }

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

    out_println!(out, "Updated task {} - {}", task.id, task.description);
    Ok(())
}
