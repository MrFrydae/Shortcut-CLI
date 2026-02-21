use std::error::Error;

use clap::Args;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    /// The project name
    #[arg(long)]
    pub name: String,

    /// The ID of the team the project belongs to
    #[arg(long)]
    pub team_id: i64,

    /// The project description
    #[arg(long)]
    pub description: Option<String>,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// The abbreviation used in Story summaries (max 3 chars)
    #[arg(long)]
    pub abbreviation: Option<String>,

    /// The number of weeks per iteration
    #[arg(long)]
    pub iteration_length: Option<i64>,

    /// An external unique ID for import tracking
    #[arg(long)]
    pub external_id: Option<String>,
}

pub async fn run(
    args: &CreateArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .parse::<api::types::CreateProjectName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateProjectDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let abbreviation = args
        .abbreviation
        .as_ref()
        .map(|a| a.parse::<api::types::CreateProjectAbbreviation>())
        .transpose()
        .map_err(|e| format!("Invalid abbreviation: {e}"))?;

    let external_id = args
        .external_id
        .as_ref()
        .map(|id| id.parse::<api::types::CreateProjectExternalId>())
        .transpose()
        .map_err(|e| format!("Invalid external ID: {e}"))?;

    if out.is_dry_run() {
        let mut body = serde_json::json!({
            "name": args.name,
            "team_id": args.team_id,
        });
        if let Some(desc) = &args.description {
            body["description"] = serde_json::json!(desc);
        }
        if let Some(color) = &args.color {
            body["color"] = serde_json::json!(color);
        }
        if let Some(abbr) = &args.abbreviation {
            body["abbreviation"] = serde_json::json!(abbr);
        }
        if let Some(iter_len) = args.iteration_length {
            body["iteration_length"] = serde_json::json!(iter_len);
        }
        if let Some(ext_id) = &args.external_id {
            body["external_id"] = serde_json::json!(ext_id);
        }
        return out.dry_run_request("POST", "/api/v3/projects", Some(&body));
    }

    let project = client
        .create_project()
        .body_map(|mut b| {
            b = b.name(name).team_id(args.team_id);
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if let Some(abbr) = abbreviation {
                b = b.abbreviation(Some(abbr));
            }
            if let Some(iter_len) = args.iteration_length {
                b = b.iteration_length(Some(iter_len));
            }
            if let Some(ext_id) = external_id {
                b = b.external_id(Some(ext_id));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create project: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*project)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        out_println!(out, "{}", project.id);
        return Ok(());
    }

    out_println!(
        out,
        "Created project {} - {} ({} stories)",
        project.id,
        project.name,
        project.stats.num_stories
    );
    Ok(())
}
