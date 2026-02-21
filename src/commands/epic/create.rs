use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;
use crate::commands::member;
use crate::output::OutputConfig;

use super::helpers::{resolve_epic_state_id, resolve_owners};
use crate::out_println;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    /// The epic's name
    #[arg(long)]
    pub name: String,

    /// The epic's description
    #[arg(long)]
    pub description: Option<String>,

    /// The epic state name or ID (e.g. "to do" or 500000010)
    #[arg(long)]
    pub state: Option<String>,

    /// The epic's deadline (RFC 3339 format, e.g. 2025-12-31T00:00:00Z)
    #[arg(long)]
    pub deadline: Option<String>,

    /// Owner(s) by @mention_name or UUID (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub owners: Vec<String>,

    /// Label names (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,

    /// Objective IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub objective_ids: Vec<i64>,

    /// Follower(s) by @mention_name or UUID (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub followers: Vec<String>,

    /// Requested-by member (@mention_name or UUID)
    #[arg(long)]
    pub requested_by: Option<String>,
}

pub async fn run(
    args: &CreateArgs,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .parse::<api::types::CreateEpicName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateEpicDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let deadline = args
        .deadline
        .as_ref()
        .map(|d| chrono::DateTime::parse_from_rfc3339(d).map(|dt| dt.with_timezone(&chrono::Utc)))
        .transpose()
        .map_err(|e| format!("Invalid deadline: {e}"))?;

    let owner_ids = resolve_owners(&args.owners, client, cache_dir).await?;
    let follower_ids = resolve_owners(&args.followers, client, cache_dir).await?;

    let requested_by_id = match &args.requested_by {
        Some(val) => Some(member::resolve_member_id(val, client, cache_dir).await?),
        None => None,
    };

    let resolved_state_id = match &args.state {
        Some(val) => Some(resolve_epic_state_id(val, client, cache_dir).await?),
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

    if out.is_dry_run() {
        let mut body = serde_json::json!({ "name": args.name });
        if let Some(desc) = &args.description {
            body["description"] = serde_json::json!(desc);
        }
        if let Some(dl) = &args.deadline {
            body["deadline"] = serde_json::json!(dl);
        }
        if let Some(state_id) = resolved_state_id {
            body["epic_state_id"] = serde_json::json!(state_id);
        }
        if !owner_ids.is_empty() {
            body["owner_ids"] = serde_json::json!(owner_ids);
        }
        if !follower_ids.is_empty() {
            body["follower_ids"] = serde_json::json!(follower_ids);
        }
        if let Some(req_id) = &requested_by_id {
            body["requested_by_id"] = serde_json::json!(req_id);
        }
        if !args.labels.is_empty() {
            body["labels"] = serde_json::json!(
                args.labels
                    .iter()
                    .map(|n| serde_json::json!({"name": n}))
                    .collect::<Vec<_>>()
            );
        }
        if !args.objective_ids.is_empty() {
            body["objective_ids"] = serde_json::json!(args.objective_ids);
        }
        return out.dry_run_request("POST", "/api/v3/epics", Some(&body));
    }

    let epic = client
        .create_epic()
        .body_map(|mut b| {
            b = b.name(name);
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(dl) = deadline {
                b = b.deadline(Some(dl));
            }
            if let Some(state_id) = resolved_state_id {
                b = b.epic_state_id(Some(state_id));
            }
            if !owner_ids.is_empty() {
                b = b.owner_ids(owner_ids);
            }
            if !follower_ids.is_empty() {
                b = b.follower_ids(follower_ids);
            }
            if let Some(req_id) = requested_by_id {
                b = b.requested_by_id(Some(req_id));
            }
            if !labels.is_empty() {
                b = b.labels(labels);
            }
            if !args.objective_ids.is_empty() {
                b = b.objective_ids(args.objective_ids.clone());
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create epic: {e}"))?;

    if out.is_quiet() {
        out_println!(out, "{}", epic.id);
        return Ok(());
    }
    out_println!(out, "Created epic {} - {}", epic.id, epic.name);
    Ok(())
}
