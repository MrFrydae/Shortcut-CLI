use crate::api;
use crate::out_println;
use crate::output::OutputConfig;
use clap::Args;
use std::error::Error;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    #[arg(long)]
    pub id: i64,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub color: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
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
        .map(|n| n.parse::<api::types::UpdateLabelName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;
    let color = args.color.clone();
    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateLabelDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    if out.is_dry_run() {
        let mut body = serde_json::Map::new();
        if let Some(name) = &args.name {
            body.insert("name".into(), serde_json::json!(name));
        }
        if let Some(color) = &args.color {
            body.insert("color".into(), serde_json::json!(color));
        }
        if let Some(desc) = &args.description {
            body.insert("description".into(), serde_json::json!(desc));
        }
        if let Some(archived) = args.archived {
            body.insert("archived".into(), serde_json::json!(archived));
        }
        let body = serde_json::Value::Object(body);
        return out.dry_run_request("PUT", &format!("/api/v3/labels/{}", args.id), Some(&body));
    }

    let label = client
        .update_label()
        .label_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(color) = color {
                b = b.color(Some(color));
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

    if out.is_quiet() {
        out_println!(out, "{}", label.id);
        return Ok(());
    }
    out_println!(out, "Updated label {} - {}", label.id, label.name);
    Ok(())
}
