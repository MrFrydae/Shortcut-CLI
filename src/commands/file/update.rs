use std::error::Error;

use clap::Args;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// The ID of the file to update
    #[arg(long)]
    pub id: i64,

    /// The new name of the file
    #[arg(long)]
    pub name: Option<String>,

    /// The new description of the file
    #[arg(long)]
    pub description: Option<String>,
}

pub async fn run(
    args: &UpdateArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if args.name.is_none() && args.description.is_none() {
        return Err("Nothing to update: pass --name and/or --description".into());
    }

    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateFileName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;
    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateFileDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    if out.is_dry_run() {
        let mut body = serde_json::Map::new();
        if let Some(name) = &args.name {
            body.insert("name".into(), serde_json::json!(name));
        }
        if let Some(desc) = &args.description {
            body.insert("description".into(), serde_json::json!(desc));
        }
        let body = serde_json::Value::Object(body);
        return out.dry_run_request("PUT", &format!("/api/v3/files/{}", args.id), Some(&body));
    }

    let file = client
        .update_file()
        .file_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update file: {}", api::format_api_error(&e)))?;

    if out.is_machine_readable() {
        out_println!(out, "{}", serde_json::to_string_pretty(&*file)?);
        return Ok(());
    }
    if out.is_quiet() {
        out_println!(out, "{}", file.id);
        return Ok(());
    }
    out_println!(out, "Updated file {} - {}", file.id, file.name);
    Ok(())
}
