use super::helpers::resolve_followers;
use crate::api;
use crate::out_println;
use crate::output::OutputConfig;
use clap::Args;
use std::error::Error;
use std::path::Path;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    #[arg(long)]
    pub id: i64,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub start_date: Option<String>,
    #[arg(long)]
    pub end_date: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub followers: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub group_ids: Vec<uuid::Uuid>,
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
        .map(|n| n.parse::<api::types::UpdateIterationName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;
    let start = args
        .start_date
        .as_ref()
        .map(|s| s.parse::<api::types::UpdateIterationStartDate>())
        .transpose()
        .map_err(|e| format!("Invalid start_date: {e}"))?;
    let end = args
        .end_date
        .as_ref()
        .map(|e_| e_.parse::<api::types::UpdateIterationEndDate>())
        .transpose()
        .map_err(|e| format!("Invalid end_date: {e}"))?;
    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateIterationDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;
    let follower_ids = resolve_followers(&args.followers, client, cache_dir).await?;
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
        let mut body = serde_json::Map::new();
        if let Some(name) = &args.name {
            body.insert("name".into(), serde_json::json!(name));
        }
        if let Some(start) = &args.start_date {
            body.insert("start_date".into(), serde_json::json!(start));
        }
        if let Some(end) = &args.end_date {
            body.insert("end_date".into(), serde_json::json!(end));
        }
        if let Some(desc) = &args.description {
            body.insert("description".into(), serde_json::json!(desc));
        }
        if !follower_ids.is_empty() {
            body.insert("follower_ids".into(), serde_json::json!(follower_ids));
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
        if !args.group_ids.is_empty() {
            body.insert("group_ids".into(), serde_json::json!(args.group_ids));
        }
        let body = serde_json::Value::Object(body);
        return out.dry_run_request(
            "PUT",
            &format!("/api/v3/iterations/{}", args.id),
            Some(&body),
        );
    }

    let iteration = client
        .update_iteration()
        .iteration_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(start) = start {
                b = b.start_date(Some(start));
            }
            if let Some(end) = end {
                b = b.end_date(Some(end));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if !follower_ids.is_empty() {
                b = b.follower_ids(follower_ids);
            }
            if !labels.is_empty() {
                b = b.labels(labels);
            }
            if !args.group_ids.is_empty() {
                b = b.group_ids(args.group_ids.clone());
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update iteration: {e}"))?;

    if out.is_quiet() {
        out_println!(out, "{}", iteration.id);
        return Ok(());
    }
    out_println!(
        out,
        "Updated iteration {} - {}",
        iteration.id,
        iteration.name
    );
    Ok(())
}
