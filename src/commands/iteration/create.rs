use super::helpers::resolve_followers;
use crate::api;
use crate::out_println;
use crate::output::OutputConfig;
use clap::Args;
use std::error::Error;
use std::path::Path;

#[derive(Args)]
pub struct CreateArgs {
    /// Launch interactive wizard to fill in fields
    #[arg(long, short = 'i')]
    pub interactive: bool,

    #[arg(long, required_unless_present = "interactive")]
    pub name: Option<String>,
    #[arg(long, required_unless_present = "interactive")]
    pub start_date: Option<String>,
    #[arg(long, required_unless_present = "interactive")]
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
    args: &CreateArgs,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let name_str = args.name.as_ref().ok_or("Name is required")?;
    let start_date_str = args.start_date.as_ref().ok_or("Start date is required")?;
    let end_date_str = args.end_date.as_ref().ok_or("End date is required")?;
    let name = name_str
        .parse::<api::types::CreateIterationName>()
        .map_err(|e| format!("Invalid name: {e}"))?;
    let start = start_date_str
        .parse::<api::types::CreateIterationStartDate>()
        .map_err(|e| format!("Invalid start_date: {e}"))?;
    let end = end_date_str
        .parse::<api::types::CreateIterationEndDate>()
        .map_err(|e| format!("Invalid end_date: {e}"))?;
    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateIterationDescription>())
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
        let mut body = serde_json::json!({
            "name": name_str,
            "start_date": start_date_str,
            "end_date": end_date_str,
        });
        if let Some(desc) = &args.description {
            body["description"] = serde_json::json!(desc);
        }
        if !follower_ids.is_empty() {
            body["follower_ids"] = serde_json::json!(follower_ids);
        }
        if !args.labels.is_empty() {
            body["labels"] = serde_json::json!(
                args.labels
                    .iter()
                    .map(|n| serde_json::json!({"name": n}))
                    .collect::<Vec<_>>()
            );
        }
        if !args.group_ids.is_empty() {
            body["group_ids"] = serde_json::json!(args.group_ids);
        }
        return out.dry_run_request("POST", "/api/v3/iterations", Some(&body));
    }

    let iteration = client
        .create_iteration()
        .body_map(|mut b| {
            b = b.name(name).start_date(start).end_date(end);
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

    if out.is_quiet() {
        out_println!(out, "{}", iteration.id);
        return Ok(());
    }
    out_println!(
        out,
        "Created iteration {} - {} ({} \u{2192} {})",
        iteration.id,
        iteration.name,
        iteration.start_date,
        iteration.end_date
    );
    Ok(())
}
