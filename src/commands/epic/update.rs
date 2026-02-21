use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;

use super::helpers::resolve_epic_state_id;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// The ID of the epic to update
    #[arg(long)]
    pub id: i64,

    /// The epic's name
    #[arg(long)]
    pub name: Option<String>,

    /// The epic's description
    #[arg(long)]
    pub description: Option<String>,

    /// The epic's deadline (RFC 3339 format, e.g. 2025-12-31T00:00:00Z)
    #[arg(long)]
    pub deadline: Option<String>,

    /// Whether the epic is archived
    #[arg(long)]
    pub archived: Option<bool>,

    /// The epic state ID or name (e.g. 500000042 or "in_progress")
    #[arg(long)]
    pub epic_state_id: Option<String>,

    /// Label names to attach (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,

    /// Objective IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub objective_ids: Vec<i64>,

    /// Owner member UUIDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub owner_ids: Vec<uuid::Uuid>,

    /// Follower member UUIDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub follower_ids: Vec<uuid::Uuid>,

    /// The UUID of the member that requested the epic
    #[arg(long)]
    pub requested_by_id: Option<uuid::Uuid>,
}

pub async fn run(
    args: &UpdateArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateEpicName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateEpicDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let deadline = args
        .deadline
        .as_ref()
        .map(|d| chrono::DateTime::parse_from_rfc3339(d).map(|dt| dt.with_timezone(&chrono::Utc)))
        .transpose()
        .map_err(|e| format!("Invalid deadline: {e}"))?;

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

    let resolved_state_id = match &args.epic_state_id {
        Some(val) => Some(resolve_epic_state_id(val, client, cache_dir).await?),
        None => None,
    };

    let epic = client
        .update_epic()
        .epic_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(dl) = deadline {
                b = b.deadline(Some(dl));
            }
            if let Some(archived) = args.archived {
                b = b.archived(Some(archived));
            }
            if let Some(state_id) = resolved_state_id {
                b = b.epic_state_id(Some(state_id));
            }
            if !labels.is_empty() {
                b = b.labels(labels);
            }
            if !args.objective_ids.is_empty() {
                b = b.objective_ids(args.objective_ids.clone());
            }
            if !args.owner_ids.is_empty() {
                b = b.owner_ids(args.owner_ids.clone());
            }
            if !args.follower_ids.is_empty() {
                b = b.follower_ids(args.follower_ids.clone());
            }
            if let Some(req_id) = args.requested_by_id {
                b = b.requested_by_id(Some(req_id));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update epic: {e}"))?;

    println!("Updated epic {} - {}", epic.id, epic.name);
    Ok(())
}
