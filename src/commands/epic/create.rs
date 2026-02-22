use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;
use crate::commands::member;
use crate::output::OutputConfig;

use super::helpers::{resolve_epic_state_id, resolve_owners};
use crate::out_println;

#[derive(Args)]
pub struct CreateArgs {
    /// Launch interactive wizard to fill in fields
    #[arg(long, short = 'i')]
    pub interactive: bool,

    /// The epic's name
    #[arg(long, required_unless_present = "interactive")]
    pub name: Option<String>,

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

    /// Team(s) (group) by @mention_name or UUID (comma-separated)
    #[arg(long = "group-id", value_delimiter = ',')]
    pub group_ids: Vec<String>,

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
    let name_str = args.name.as_ref().ok_or("Name is required")?;
    let name = name_str
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

    let resolved_group_ids = {
        let mut ids = Vec::with_capacity(args.group_ids.len());
        for g in &args.group_ids {
            ids.push(
                crate::commands::group::helpers::resolve_group_id(g, client, cache_dir).await?,
            );
        }
        ids
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
        let mut body = serde_json::json!({ "name": name_str });
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
        if !resolved_group_ids.is_empty() {
            body["group_ids"] = serde_json::json!(resolved_group_ids);
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
            if !resolved_group_ids.is_empty() {
                b = b.group_ids(resolved_group_ids);
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
