use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;

use super::helpers::resolve_followers;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// The ID of the iteration to update
    #[arg(long)]
    pub id: i64,

    /// The iteration name
    #[arg(long)]
    pub name: Option<String>,

    /// The start date (YYYY-MM-DD)
    #[arg(long)]
    pub start_date: Option<String>,

    /// The end date (YYYY-MM-DD)
    #[arg(long)]
    pub end_date: Option<String>,

    /// The iteration description
    #[arg(long)]
    pub description: Option<String>,

    /// Follower(s) by @mention_name or UUID (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub followers: Vec<String>,

    /// Label names (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,

    /// Group IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub group_ids: Vec<uuid::Uuid>,
}

pub async fn run(
    args: &UpdateArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    if let Some(ref sd) = args.start_date {
        chrono::NaiveDate::parse_from_str(sd, "%Y-%m-%d")
            .map_err(|e| format!("Invalid start date format (expected YYYY-MM-DD): {e}"))?;
    }
    if let Some(ref ed) = args.end_date {
        chrono::NaiveDate::parse_from_str(ed, "%Y-%m-%d")
            .map_err(|e| format!("Invalid end date format (expected YYYY-MM-DD): {e}"))?;
    }

    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateIterationName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateIterationDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let start_date = args
        .start_date
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateIterationStartDate>())
        .transpose()
        .map_err(|e| format!("Invalid start date: {e}"))?;

    let end_date = args
        .end_date
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateIterationEndDate>())
        .transpose()
        .map_err(|e| format!("Invalid end date: {e}"))?;

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

    let iteration = client
        .update_iteration()
        .iteration_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(sd) = start_date {
                b = b.start_date(Some(sd));
            }
            if let Some(ed) = end_date {
                b = b.end_date(Some(ed));
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

    println!("Updated iteration {} - {}", iteration.id, iteration.name);
    Ok(())
}
