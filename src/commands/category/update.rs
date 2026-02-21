use std::error::Error;

use clap::Args;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// The ID of the category to update
    #[arg(long)]
    pub id: i64,

    /// The category name
    #[arg(long)]
    pub name: Option<String>,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// Whether the category is archived
    #[arg(long)]
    pub archived: Option<bool>,
}

pub async fn run(
    args: &UpdateArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateCategoryName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let category = client
        .update_category()
        .category_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if let Some(archived) = args.archived {
                b = b.archived(Some(archived));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update category: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*category)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        out_println!(out, "{}", category.id);
        return Ok(());
    }

    out_println!(out, "Updated category {} - {}", category.id, category.name);
    Ok(())
}
