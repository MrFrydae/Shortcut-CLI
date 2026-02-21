use std::error::Error;

use clap::Args;

use crate::api;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// The ID of the label to update
    #[arg(long)]
    pub id: i64,

    /// The label name
    #[arg(long)]
    pub name: Option<String>,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// The label description
    #[arg(long)]
    pub description: Option<String>,

    /// Whether the label is archived
    #[arg(long)]
    pub archived: Option<bool>,
}

pub async fn run(args: &UpdateArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateLabelName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateLabelDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let label = client
        .update_label()
        .label_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(archived) = args.archived {
                b = b.archived(Some(archived));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update label: {e}"))?;

    println!("Updated label {} - {}", label.id, label.name);
    Ok(())
}
