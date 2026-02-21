use std::error::Error;

use clap::Args;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// The ID of the project to update
    #[arg(long)]
    pub id: i64,

    /// The project name
    #[arg(long)]
    pub name: Option<String>,

    /// The project description
    #[arg(long)]
    pub description: Option<String>,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// The abbreviation used in Story summaries (max 3 chars)
    #[arg(long)]
    pub abbreviation: Option<String>,

    /// Whether the project is archived
    #[arg(long)]
    pub archived: Option<bool>,

    /// The ID of the team the project belongs to
    #[arg(long)]
    pub team_id: Option<i64>,

    /// The number of days before the thermometer appears
    #[arg(long)]
    pub days_to_thermometer: Option<i64>,

    /// Enable or disable thermometers in the Story summary
    #[arg(long)]
    pub show_thermometer: Option<bool>,
}

pub async fn run(
    args: &UpdateArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateProjectName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateProjectDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let project = client
        .update_project()
        .project_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if let Some(abbr) = &args.abbreviation {
                b = b.abbreviation(Some(abbr.clone()));
            }
            if let Some(archived) = args.archived {
                b = b.archived(Some(archived));
            }
            if let Some(team_id) = args.team_id {
                b = b.team_id(Some(team_id));
            }
            if let Some(days) = args.days_to_thermometer {
                b = b.days_to_thermometer(Some(days));
            }
            if let Some(show) = args.show_thermometer {
                b = b.show_thermometer(Some(show));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update project: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*project)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        out_println!(out, "{}", project.id);
        return Ok(());
    }

    out_println!(out, "Updated project {} - {}", project.id, project.name);
    Ok(())
}
