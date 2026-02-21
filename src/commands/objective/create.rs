use std::error::Error;

use clap::Args;

use crate::api;

use super::helpers::build_categories;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    /// The objective name
    #[arg(long)]
    pub name: String,

    /// The objective description
    #[arg(long)]
    pub description: Option<String>,

    /// The state ("to do", "in progress", "done")
    #[arg(long)]
    pub state: Option<String>,

    /// Category names (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub categories: Vec<String>,
}

pub async fn run(args: &CreateArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .parse::<api::types::CreateObjectiveName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateObjectiveDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let state = args
        .state
        .as_ref()
        .map(|s| s.parse::<api::types::CreateObjectiveState>())
        .transpose()
        .map_err(|e| format!("Invalid state: {e}"))?;

    let categories = build_categories(&args.categories)?;

    let objective = client
        .create_objective()
        .body_map(|mut b| {
            b = b.name(name);
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(state) = state {
                b = b.state(Some(state));
            }
            if !categories.is_empty() {
                b = b.categories(categories);
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create objective: {e}"))?;

    println!(
        "Created objective {} - {} ({})",
        objective.id, objective.name, objective.state
    );
    Ok(())
}
