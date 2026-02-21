use std::error::Error;

use clap::Args;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

#[derive(Args)]
pub struct WorkflowArgs {
    /// List all workflows
    #[arg(long)]
    pub list: bool,

    /// Get a specific workflow by ID
    #[arg(long)]
    pub id: Option<i64>,
}

pub async fn run(
    args: &WorkflowArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if args.list {
        let workflows = client
            .list_workflows()
            .send()
            .await
            .map_err(|e| format!("Failed to list workflows: {e}"))?;
        for wf in workflows.iter() {
            out_println!(out, "{} - {}", wf.id, wf.name);
        }
    } else if let Some(id) = args.id {
        let wf = client
            .get_workflow()
            .workflow_public_id(id)
            .send()
            .await
            .map_err(|e| format!("Failed to get workflow: {e}"))?;
        out_println!(out, "{} (id: {})\n", wf.name, wf.id);
        let mut states: Vec<_> = wf.states.iter().collect();
        states.sort_by_key(|s| s.position);
        let id_width = states
            .iter()
            .map(|s| s.id.to_string().len())
            .max()
            .unwrap_or(0)
            .max(2);
        let type_width = states
            .iter()
            .map(|s| s.type_.len())
            .max()
            .unwrap_or(0)
            .max(4);
        out_println!(
            out,
            "  {:<id_w$}  {:<type_w$}  Name",
            "ID",
            "Type",
            id_w = id_width,
            type_w = type_width
        );
        for state in &states {
            out_println!(
                out,
                "  {:<id_w$}  {:<type_w$}  {}",
                state.id,
                state.type_,
                state.name,
                id_w = id_width,
                type_w = type_width,
            );
        }
    }
    Ok(())
}
