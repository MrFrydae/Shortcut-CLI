use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;

use super::helpers::{normalize_name, read_cache, write_cache};

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    /// The label name
    #[arg(long)]
    pub name: String,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// The label description
    #[arg(long)]
    pub description: Option<String>,
}

pub async fn run(
    args: &CreateArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .parse::<api::types::CreateLabelParamsName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateLabelParamsDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let label = client
        .create_label()
        .body_map(|mut b| {
            b = b.name(name);
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create label: {e}"))?;

    let color = label
        .color
        .as_deref()
        .map(|c| format!(" ({c})"))
        .unwrap_or_default();
    println!("Created label {} - {}{}", label.id, label.name, color);

    // Update cache with new label
    let mut map = read_cache(cache_dir).unwrap_or_default();
    map.insert(normalize_name(&label.name), label.id);
    write_cache(&map, cache_dir);

    Ok(())
}
