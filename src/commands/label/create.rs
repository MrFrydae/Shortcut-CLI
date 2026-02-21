use crate::api;
use crate::out_println;
use crate::output::OutputConfig;
use clap::Args;
use std::error::Error;
use std::path::Path;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub color: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
}

pub async fn run(
    args: &CreateArgs,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .parse::<api::types::CreateLabelParamsName>()
        .map_err(|e| format!("Invalid name: {e}"))?;
    let color = args.color.clone();
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
            if let Some(color) = color {
                b = b.color(Some(color));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create label: {e}"))?;

    // Update cache with new label
    let mut cache = super::helpers::read_cache(cache_dir).unwrap_or_default();
    cache.insert(super::helpers::normalize_name(&label.name), label.id);
    super::helpers::write_cache(&cache, cache_dir);

    if out.is_quiet() {
        out_println!(out, "{}", label.id);
        return Ok(());
    }
    let color_str = label
        .color
        .as_deref()
        .map(|c| format!(" ({c})"))
        .unwrap_or_default();
    out_println!(
        out,
        "Created label {} - {}{}",
        label.id,
        label.name,
        color_str
    );
    Ok(())
}
