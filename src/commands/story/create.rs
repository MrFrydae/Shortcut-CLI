use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;
use crate::output::OutputConfig;

use super::helpers::{resolve_custom_field_args, resolve_owners, resolve_workflow_state_id};
use crate::out_println;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    /// The name of the story
    #[arg(long)]
    pub name: String,

    /// The description of the story
    #[arg(long)]
    pub description: Option<String>,

    /// The type of story (feature, bug, chore)
    #[arg(long, name = "type")]
    pub story_type: Option<String>,

    /// Owner(s) by @mention_name or UUID (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub owner: Vec<String>,

    /// The workflow state name or ID
    #[arg(long)]
    pub state: Option<String>,

    /// The epic ID to associate with
    #[arg(long)]
    pub epic_id: Option<i64>,

    /// The story point estimate
    #[arg(long)]
    pub estimate: Option<i64>,

    /// Label names (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,

    /// The iteration ID to assign this story to
    #[arg(long)]
    pub iteration_id: Option<i64>,

    /// Set a custom field value (format: "FieldName=Value", repeatable)
    #[arg(long = "custom-field")]
    pub custom_fields: Vec<String>,
}

pub async fn run(
    args: &CreateArgs,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .parse::<api::types::CreateStoryParamsName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateStoryParamsDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let story_type = args
        .story_type
        .as_ref()
        .map(|t| t.parse::<api::types::CreateStoryParamsStoryType>())
        .transpose()
        .map_err(|e| format!("Invalid story type: {e}"))?;

    let owner_ids = resolve_owners(&args.owner, client, cache_dir).await?;

    let resolved_state_id = match &args.state {
        Some(val) => Some(resolve_workflow_state_id(val, client, cache_dir).await?),
        None => None,
    };

    let labels: Vec<api::types::CreateLabelParams> = args
        .labels
        .iter()
        .map(|n| -> Result<_, String> {
            Ok(api::types::CreateLabelParams {
                name: n.parse().map_err(|e| format!("Invalid label name: {e}"))?,
                color: None,
                description: None,
                external_id: None,
            })
        })
        .collect::<Result<_, _>>()?;

    let custom_field_params =
        resolve_custom_field_args(&args.custom_fields, client, cache_dir).await?;

    if out.is_dry_run() {
        let mut body = serde_json::json!({ "name": args.name });
        if let Some(desc) = &args.description {
            body["description"] = serde_json::json!(desc);
        }
        if let Some(st) = &args.story_type {
            body["story_type"] = serde_json::json!(st);
        }
        if !owner_ids.is_empty() {
            body["owner_ids"] = serde_json::json!(owner_ids);
        }
        if let Some(state_id) = resolved_state_id {
            body["workflow_state_id"] = serde_json::json!(state_id);
        }
        if let Some(epic_id) = args.epic_id {
            body["epic_id"] = serde_json::json!(epic_id);
        }
        if let Some(estimate) = args.estimate {
            body["estimate"] = serde_json::json!(estimate);
        }
        if !args.labels.is_empty() {
            body["labels"] = serde_json::json!(
                args.labels
                    .iter()
                    .map(|n| serde_json::json!({"name": n}))
                    .collect::<Vec<_>>()
            );
        }
        if let Some(iter_id) = args.iteration_id {
            body["iteration_id"] = serde_json::json!(iter_id);
        }
        if !custom_field_params.is_empty() {
            body["custom_fields"] = serde_json::json!(custom_field_params);
        }
        return out.dry_run_request("POST", "/api/v3/stories", Some(&body));
    }

    let story = client
        .create_story()
        .body_map(|mut b| {
            b = b.name(name);
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(st) = story_type {
                b = b.story_type(Some(st));
            }
            if !owner_ids.is_empty() {
                b = b.owner_ids(Some(owner_ids));
            }
            if let Some(state_id) = resolved_state_id {
                b = b.workflow_state_id(Some(state_id));
            }
            if let Some(epic_id) = args.epic_id {
                b = b.epic_id(Some(epic_id));
            }
            if let Some(estimate) = args.estimate {
                b = b.estimate(Some(estimate));
            }
            if !labels.is_empty() {
                b = b.labels(labels);
            }
            if let Some(iter_id) = args.iteration_id {
                b = b.iteration_id(Some(iter_id));
            }
            if !custom_field_params.is_empty() {
                b = b.custom_fields(custom_field_params);
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create story: {e}"))?;

    if out.is_json() {
        out_println!(
            out,
            "{}",
            serde_json::json!({"id": story.id, "name": story.name})
        );
        return Ok(());
    }
    if out.is_quiet() {
        out_println!(out, "{}", story.id);
        return Ok(());
    }
    out_println!(out, "Created story {} - {}", story.id, story.name);
    Ok(())
}
