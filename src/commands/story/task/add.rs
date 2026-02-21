use std::error::Error;

use clap::Args;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct AddArgs {
    /// The story ID
    #[arg(long)]
    pub story_id: i64,
    /// Task description(s) — repeat for multiple tasks
    #[arg(long)]
    pub description: Vec<String>,
}

pub async fn run(
    args: &AddArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if args.description.is_empty() {
        return Err("At least one --description is required".into());
    }

    let descriptions: Vec<api::types::CreateTaskDescription> = args
        .description
        .iter()
        .map(|d| {
            d.parse::<api::types::CreateTaskDescription>()
                .map_err(|e| format!("Invalid description '{}': {e}", d))
        })
        .collect::<Result<_, _>>()?;

    if out.is_dry_run() {
        for desc in &args.description {
            let body = serde_json::json!({ "description": desc });
            out.dry_run_request(
                "POST",
                &format!("/api/v3/stories/{}/tasks", args.story_id),
                Some(&body),
            )?;
        }
        return Ok(());
    }

    let mut errors = Vec::new();
    for desc in descriptions {
        match client
            .create_task()
            .story_public_id(args.story_id)
            .body_map(|b| b.description(desc.clone()))
            .send()
            .await
        {
            Ok(task) => out_println!(out, "Created task {} - {}", task.id, task.description),
            Err(e) => errors.push(format!("Failed to create task '{}': {e}", &*desc)),
        }
    }

    if !errors.is_empty() {
        return Err(errors.join("\n").into());
    }
    Ok(())
}
