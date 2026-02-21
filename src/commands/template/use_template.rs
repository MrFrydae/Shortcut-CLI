use std::error::Error;
use std::path::{Path, PathBuf};

use clap::Args;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

use crate::commands::story::helpers::{
    resolve_custom_field_args, resolve_owners, resolve_workflow_state_id,
};

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UseArgs {
    /// The entity template UUID
    #[arg(long)]
    pub id: String,

    /// The name of the new story (required)
    #[arg(long)]
    pub name: String,

    /// Override the description from the template
    #[arg(long, conflicts_with = "description_file")]
    pub description: Option<String>,

    /// Read the description override from a file
    #[arg(long)]
    pub description_file: Option<PathBuf>,

    /// Override the story type (feature, bug, chore)
    #[arg(long, name = "type")]
    pub story_type: Option<String>,

    /// Override owner(s) by @mention_name or UUID (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub owner: Vec<String>,

    /// Override the workflow state name or ID
    #[arg(long)]
    pub state: Option<String>,

    /// Override the epic ID
    #[arg(long)]
    pub epic_id: Option<i64>,

    /// Override the story point estimate
    #[arg(long)]
    pub estimate: Option<i64>,

    /// Override label names (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,

    /// Override the iteration ID
    #[arg(long)]
    pub iteration_id: Option<i64>,

    /// Set a custom field value (format: "FieldName=Value", repeatable)
    #[arg(long = "custom-field")]
    pub custom_fields: Vec<String>,
}

pub async fn run(
    args: &UseArgs,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let template_uuid: uuid::Uuid = args
        .id
        .parse()
        .map_err(|e| format!("Invalid entity template UUID: {e}"))?;

    // Fetch the template
    let template = client
        .get_entity_template()
        .entity_template_public_id(template_uuid)
        .send()
        .await
        .map_err(|e| format!("Failed to get entity template: {e}"))?;

    let template_name = template.name.clone();
    let sc = &template.story_contents;

    // Resolve description: CLI flag wins over template value
    let description = if let Some(path) = &args.description_file {
        Some(
            std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read file '{}': {e}", path.display()))?,
        )
    } else if args.description.is_some() {
        args.description.clone()
    } else {
        sc.description.clone()
    };

    let description = description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateStoryParamsDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    // Story type: CLI flag wins
    let story_type_str = args.story_type.as_ref().or(sc.story_type.as_ref());
    let story_type = story_type_str
        .map(|t| t.parse::<api::types::CreateStoryParamsStoryType>())
        .transpose()
        .map_err(|e| format!("Invalid story type: {e}"))?;

    // Owners: CLI flag wins (non-empty means override)
    let owner_ids = if !args.owner.is_empty() {
        resolve_owners(&args.owner, client, cache_dir).await?
    } else {
        sc.owner_ids.clone()
    };

    // Workflow state: CLI flag wins
    let resolved_state_id = if let Some(val) = &args.state {
        Some(resolve_workflow_state_id(val, client, cache_dir).await?)
    } else {
        sc.workflow_state_id
    };

    // Epic: CLI flag wins
    let epic_id = args.epic_id.or(sc.epic_id);

    // Estimate: CLI flag wins
    let estimate = args.estimate.or(sc.estimate);

    // Iteration: CLI flag wins
    let iteration_id = args.iteration_id.or(sc.iteration_id);

    // Labels: CLI flag wins (non-empty means override), else convert template labels
    let labels: Vec<api::types::CreateLabelParams> = if !args.labels.is_empty() {
        args.labels
            .iter()
            .map(|n| -> Result<_, String> {
                Ok(api::types::CreateLabelParams {
                    name: n.parse().map_err(|e| format!("Invalid label name: {e}"))?,
                    color: None,
                    description: None,
                    external_id: None,
                })
            })
            .collect::<Result<_, _>>()?
    } else {
        sc.labels
            .iter()
            .map(|l| -> Result<_, String> {
                Ok(api::types::CreateLabelParams {
                    name: l
                        .name
                        .parse()
                        .map_err(|e| format!("Invalid label name: {e}"))?,
                    color: None,
                    description: None,
                    external_id: None,
                })
            })
            .collect::<Result<_, _>>()?
    };

    // Custom fields: CLI flag wins (non-empty means override)
    let custom_field_params = if !args.custom_fields.is_empty() {
        resolve_custom_field_args(&args.custom_fields, client, cache_dir).await?
    } else {
        sc.custom_fields.clone()
    };

    // Build the story name
    let name = args
        .name
        .parse::<api::types::CreateStoryParamsName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    if out.is_dry_run() {
        let mut body = serde_json::json!({
            "name": args.name,
            "story_template_id": template_uuid.to_string(),
        });
        if let Some(desc) = &args.description {
            body["description"] = serde_json::json!(desc);
        }
        if let Some(st) = story_type_str {
            body["story_type"] = serde_json::json!(st);
        }
        if !owner_ids.is_empty() {
            body["owner_ids"] = serde_json::json!(owner_ids);
        }
        if let Some(state_id) = resolved_state_id {
            body["workflow_state_id"] = serde_json::json!(state_id);
        }
        if let Some(epic_id) = epic_id {
            body["epic_id"] = serde_json::json!(epic_id);
        }
        if let Some(estimate) = estimate {
            body["estimate"] = serde_json::json!(estimate);
        }
        if !labels.is_empty() {
            body["labels"] = serde_json::json!(
                labels
                    .iter()
                    .map(|l| serde_json::json!({"name": &*l.name}))
                    .collect::<Vec<_>>()
            );
        }
        if let Some(iter_id) = iteration_id {
            body["iteration_id"] = serde_json::json!(iter_id);
        }
        if !custom_field_params.is_empty() {
            body["custom_fields"] = serde_json::json!(custom_field_params);
        }
        return out.dry_run_request("POST", "/api/v3/stories", Some(&body));
    }

    // Create the story with merged fields
    let story = client
        .create_story()
        .body_map(|mut b| {
            b = b.name(name);
            b = b.story_template_id(Some(template_uuid));
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
            if let Some(epic_id) = epic_id {
                b = b.epic_id(Some(epic_id));
            }
            if let Some(estimate) = estimate {
                b = b.estimate(Some(estimate));
            }
            if !labels.is_empty() {
                b = b.labels(labels);
            }
            if let Some(iter_id) = iteration_id {
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
    out_println!(
        out,
        "Created story {} - {} (from template \"{}\")",
        story.id,
        story.name,
        template_name
    );
    Ok(())
}
