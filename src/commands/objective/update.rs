use std::error::Error;

use clap::Args;

use crate::api;
use crate::output::OutputConfig;

use super::helpers::build_categories;
use crate::out_println;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// The ID of the objective to update
    #[arg(long)]
    pub id: i64,

    /// The objective name
    #[arg(long)]
    pub name: Option<String>,

    /// The objective description
    #[arg(long)]
    pub description: Option<String>,

    /// The state ("to do", "in progress", "done")
    #[arg(long)]
    pub state: Option<String>,

    /// Whether the objective is archived
    #[arg(long)]
    pub archived: Option<bool>,

    /// Category names (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub categories: Vec<String>,
}

pub async fn run(
    args: &UpdateArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateObjectiveName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateObjectiveDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let state = args
        .state
        .as_ref()
        .map(|s| s.parse::<api::types::UpdateObjectiveState>())
        .transpose()
        .map_err(|e| format!("Invalid state: {e}"))?;

    let categories = build_categories(&args.categories)?;

    let objective = client
        .update_objective()
        .objective_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(state) = state {
                b = b.state(Some(state));
            }
            if let Some(archived) = args.archived {
                b = b.archived(Some(archived));
            }
            if !categories.is_empty() {
                b = b.categories(categories);
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update objective: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*objective)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        out_println!(out, "{}", objective.id);
        return Ok(());
    }

    out_println!(
        out,
        "Updated objective {} - {}",
        objective.id,
        objective.name
    );
    Ok(())
}
