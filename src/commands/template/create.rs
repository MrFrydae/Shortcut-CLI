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
pub struct CreateArgs {
    /// The name of the template
    #[arg(long)]
    pub name: String,

    /// The default story name in the template
    #[arg(long)]
    pub story_name: Option<String>,

    /// The default story description
    #[arg(long, conflicts_with = "description_file")]
    pub description: Option<String>,

    /// Read the default description from a file
    #[arg(long)]
    pub description_file: Option<PathBuf>,

    /// The default story type (feature, bug, chore)
    #[arg(long, name = "type")]
    pub story_type: Option<String>,

    /// Default owner(s) by @mention_name or UUID (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub owner: Vec<String>,

    /// The default workflow state name or ID
    #[arg(long)]
    pub state: Option<String>,

    /// The default epic ID
    #[arg(long)]
    pub epic_id: Option<i64>,

    /// The default story point estimate
    #[arg(long)]
    pub estimate: Option<i64>,

    /// Default label names (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,

    /// The default iteration ID
    #[arg(long)]
    pub iteration_id: Option<i64>,

    /// Set a default custom field value (format: "FieldName=Value", repeatable)
    #[arg(long = "custom-field")]
    pub custom_fields: Vec<String>,
}

pub async fn run(
    args: &CreateArgs,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let template_name = args
        .name
        .parse::<api::types::CreateEntityTemplateName>()
        .map_err(|e| format!("Invalid template name: {e}"))?;

    let description = if let Some(path) = &args.description_file {
        Some(
            std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read file '{}': {e}", path.display()))?,
        )
    } else {
        args.description.clone()
    };

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
        let mut sc = serde_json::Map::new();
        if let Some(sn) = &args.story_name {
            sc.insert("name".into(), serde_json::json!(sn));
        }
        if let Some(desc) = &description {
            sc.insert("description".into(), serde_json::json!(desc));
        }
        if let Some(st) = &args.story_type {
            sc.insert("story_type".into(), serde_json::json!(st));
        }
        if !owner_ids.is_empty() {
            sc.insert("owner_ids".into(), serde_json::json!(owner_ids));
        }
        if let Some(state_id) = resolved_state_id {
            sc.insert("workflow_state_id".into(), serde_json::json!(state_id));
        }
        if let Some(epic_id) = args.epic_id {
            sc.insert("epic_id".into(), serde_json::json!(epic_id));
        }
        if let Some(estimate) = args.estimate {
            sc.insert("estimate".into(), serde_json::json!(estimate));
        }
        if !args.labels.is_empty() {
            sc.insert(
                "labels".into(),
                serde_json::json!(
                    args.labels
                        .iter()
                        .map(|n| serde_json::json!({"name": n}))
                        .collect::<Vec<_>>()
                ),
            );
        }
        if let Some(iter_id) = args.iteration_id {
            sc.insert("iteration_id".into(), serde_json::json!(iter_id));
        }
        if !custom_field_params.is_empty() {
            sc.insert(
                "custom_fields".into(),
                serde_json::json!(custom_field_params),
            );
        }
        let body = serde_json::json!({
            "name": args.name,
            "story_contents": serde_json::Value::Object(sc),
        });
        return out.dry_run_request("POST", "/api/v3/entity-templates", Some(&body));
    }

    let story_name = args
        .story_name
        .as_ref()
        .map(|n| n.parse::<api::types::CreateStoryContentsName>())
        .transpose()
        .map_err(|e| format!("Invalid story name: {e}"))?;

    let mut contents = api::types::CreateStoryContents::default();

    if let Some(name) = story_name {
        contents.name = Some(name);
    }
    if let Some(desc) = description {
        contents.description = Some(desc);
    }
    if let Some(st) = &args.story_type {
        contents.story_type = Some(st.clone());
    }
    if !owner_ids.is_empty() {
        contents.owner_ids = owner_ids;
    }
    if let Some(state_id) = resolved_state_id {
        contents.workflow_state_id = Some(state_id);
    }
    if let Some(epic_id) = args.epic_id {
        contents.epic_id = Some(epic_id);
    }
    if let Some(estimate) = args.estimate {
        contents.estimate = Some(estimate);
    }
    if !labels.is_empty() {
        contents.labels = labels;
    }
    if let Some(iter_id) = args.iteration_id {
        contents.iteration_id = Some(iter_id);
    }
    if !custom_field_params.is_empty() {
        contents.custom_fields = custom_field_params;
    }

    let template = client
        .create_entity_template()
        .body_map(|b| b.name(template_name).story_contents(contents))
        .send()
        .await
        .map_err(|e| format!("Failed to create entity template: {e}"))?;

    if out.is_json() {
        out_println!(
            out,
            "{}",
            serde_json::json!({"id": template.id, "name": template.name})
        );
        return Ok(());
    }
    if out.is_quiet() {
        out_println!(out, "{}", template.id);
        return Ok(());
    }
    out_println!(out, "Created template {} - {}", template.id, template.name);
    Ok(())
}
