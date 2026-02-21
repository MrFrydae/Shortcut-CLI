use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;

use super::helpers::resolve_followers;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    /// The iteration name
    #[arg(long)]
    pub name: String,

    /// The start date (YYYY-MM-DD)
    #[arg(long)]
    pub start_date: String,

    /// The end date (YYYY-MM-DD)
    #[arg(long)]
    pub end_date: String,

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
    args: &CreateArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    chrono::NaiveDate::parse_from_str(&args.start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start date format (expected YYYY-MM-DD): {e}"))?;
    chrono::NaiveDate::parse_from_str(&args.end_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end date format (expected YYYY-MM-DD): {e}"))?;

    let name = args
        .name
        .parse::<api::types::CreateIterationName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateIterationDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let start_date = args
        .start_date
        .parse::<api::types::CreateIterationStartDate>()
        .map_err(|e| format!("Invalid start date: {e}"))?;

    let end_date = args
        .end_date
        .parse::<api::types::CreateIterationEndDate>()
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
        .create_iteration()
        .body_map(|mut b| {
            b = b.name(name).start_date(start_date).end_date(end_date);
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
        .map_err(|e| format!("Failed to create iteration: {e}"))?;

    println!(
        "Created iteration {} - {} ({} \u{2192} {})",
        iteration.id, iteration.name, iteration.start_date, iteration.end_date
    );
    Ok(())
}
