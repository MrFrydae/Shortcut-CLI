use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;
use crate::commands::member;
use crate::output::OutputConfig;

use super::helpers::{
    normalize_name, resolve_epic_state_id, resolve_epic_state_name, resolve_owners,
};
use crate::out_println;

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

    /// Owner(s) by @mention_name or UUID (comma-separated, replaces all)
    #[arg(long, value_delimiter = ',')]
    pub owner: Vec<String>,

    /// Add owner(s) to the existing list (comma-separated @mention_name or UUID)
    #[arg(long, value_delimiter = ',', conflicts_with = "owner")]
    pub add_owner: Vec<String>,

    /// Follower(s) by @mention_name or UUID (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub follower: Vec<String>,

    /// Requested-by member (@mention_name or UUID)
    #[arg(long)]
    pub requested_by: Option<String>,

    /// Skip update if the epic is currently in any of these states (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub unless_state: Vec<String>,
}

pub async fn run(
    args: &UpdateArgs,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
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

    let mut owner_ids = resolve_owners(&args.owner, client, cache_dir).await?;
    let add_owner_ids = resolve_owners(&args.add_owner, client, cache_dir).await?;
    let follower_ids = resolve_owners(&args.follower, client, cache_dir).await?;
    let requested_by_id = match &args.requested_by {
        Some(val) => Some(member::resolve_member_id(val, client, cache_dir).await?),
        None => None,
    };

    // Fetch epic if needed for --add-owner merge or --unless-state check
    let need_fetch = !add_owner_ids.is_empty() || !args.unless_state.is_empty();
    let fetched_epic = if need_fetch {
        Some(
            client
                .get_epic()
                .epic_public_id(args.id)
                .send()
                .await
                .map_err(|e| {
                    format!("Failed to fetch epic: {}", crate::api::format_api_error(&e))
                })?,
        )
    } else {
        None
    };

    // Merge --add-owner into existing owners
    if !add_owner_ids.is_empty()
        && let Some(epic) = &fetched_epic
    {
        let mut merged: Vec<uuid::Uuid> = epic.owner_ids.clone();
        for id in &add_owner_ids {
            if !merged.contains(id) {
                merged.push(*id);
            }
        }
        owner_ids = merged;
    }

    // --- unless-state guard (before state resolution to avoid cache artifacts) ---
    if !args.unless_state.is_empty() {
        let epic = fetched_epic.as_ref().unwrap();

        let current_name = resolve_epic_state_name(epic.epic_state_id, client, cache_dir).await;

        let normalized_current = normalize_name(&current_name);
        for excluded in &args.unless_state {
            if normalize_name(excluded) == normalized_current {
                return output_skipped(args.id, &current_name, excluded, out);
            }
        }
    }

    let resolved_state_id = match &args.epic_state_id {
        Some(val) => Some(resolve_epic_state_id(val, client, cache_dir).await?),
        None => None,
    };

    if out.is_dry_run() {
        let mut body = serde_json::Map::new();
        if let Some(name) = &args.name {
            body.insert("name".into(), serde_json::json!(name));
        }
        if let Some(desc) = &args.description {
            body.insert("description".into(), serde_json::json!(desc));
        }
        if let Some(dl) = &args.deadline {
            body.insert("deadline".into(), serde_json::json!(dl));
        }
        if let Some(archived) = args.archived {
            body.insert("archived".into(), serde_json::json!(archived));
        }
        if let Some(state_id) = resolved_state_id {
            body.insert("epic_state_id".into(), serde_json::json!(state_id));
        }
        if !args.labels.is_empty() {
            body.insert(
                "labels".into(),
                serde_json::json!(
                    args.labels
                        .iter()
                        .map(|n| serde_json::json!({"name": n}))
                        .collect::<Vec<_>>()
                ),
            );
        }
        if !args.objective_ids.is_empty() {
            body.insert(
                "objective_ids".into(),
                serde_json::json!(args.objective_ids),
            );
        }
        if !owner_ids.is_empty() {
            body.insert("owner_ids".into(), serde_json::json!(owner_ids));
        }
        if !follower_ids.is_empty() {
            body.insert("follower_ids".into(), serde_json::json!(follower_ids));
        }
        if let Some(req_id) = &requested_by_id {
            body.insert("requested_by_id".into(), serde_json::json!(req_id));
        }
        let body = serde_json::Value::Object(body);
        return out.dry_run_request("PUT", &format!("/api/v3/epics/{}", args.id), Some(&body));
    }

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
            if !owner_ids.is_empty() {
                b = b.owner_ids(owner_ids.clone());
            }
            if !follower_ids.is_empty() {
                b = b.follower_ids(follower_ids.clone());
            }
            if let Some(req_id) = requested_by_id {
                b = b.requested_by_id(Some(req_id));
            }
            b
        })
        .send()
        .await
        .map_err(|e| {
            format!(
                "Failed to update epic: {}",
                crate::api::format_api_error(&e)
            )
        })?;

    if out.is_quiet() {
        out_println!(out, "{}", epic.id);
        return Ok(());
    }
    out_println!(out, "Updated epic {} - {}", epic.id, epic.name);
    Ok(())
}

fn output_skipped(
    id: i64,
    current_state: &str,
    matched_arg: &str,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if out.is_json() {
        out_println!(
            out,
            "{}",
            serde_json::json!({
                "id": id,
                "skipped": true,
                "current_state": current_state,
                "reason": format!(
                    "epic is in '{}' (matches --unless-state '{}')",
                    current_state, matched_arg
                )
            })
        );
    } else if !out.is_quiet() {
        out_println!(
            out,
            "Skipped: epic {} is in '{}' (matches --unless-state '{}')",
            id,
            current_state,
            matched_arg
        );
    }
    Ok(())
}
