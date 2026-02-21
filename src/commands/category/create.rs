use std::error::Error;

use clap::Args;

use crate::api;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    /// The category name
    #[arg(long)]
    pub name: String,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// The category type (e.g. "milestone")
    #[arg(long, name = "type")]
    pub category_type: Option<String>,

    /// An external unique ID for import tracking
    #[arg(long)]
    pub external_id: Option<String>,
}

pub async fn run(args: &CreateArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .parse::<api::types::CreateCategoryName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let external_id = args
        .external_id
        .as_ref()
        .map(|id| id.parse::<api::types::CreateCategoryExternalId>())
        .transpose()
        .map_err(|e| format!("Invalid external ID: {e}"))?;

    let category = client
        .create_category()
        .body_map(|mut b| {
            b = b.name(name);
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if let Some(t) = &args.category_type {
                b = b.type_(Some(serde_json::Value::String(t.clone())));
            }
            if let Some(ext_id) = external_id {
                b = b.external_id(Some(ext_id));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create category: {e}"))?;

    let color = category.color.as_deref().unwrap_or("none");
    println!(
        "Created category {} - {} ({})",
        category.id, category.name, color
    );
    Ok(())
}
