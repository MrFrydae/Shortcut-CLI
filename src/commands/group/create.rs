use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;
use crate::output::OutputConfig;

use super::helpers::resolve_members;
use crate::out_println;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    /// The group name
    #[arg(long)]
    pub name: String,

    /// The @mention name for the group
    #[arg(long)]
    pub mention_name: String,

    /// The group description
    #[arg(long)]
    pub description: Option<String>,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// Member(s) by @mention_name or UUID (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub member_ids: Vec<String>,

    /// Workflow IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub workflow_ids: Vec<i64>,
}

pub async fn run(
    args: &CreateArgs,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .parse::<api::types::CreateGroupName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let mention_name = args
        .mention_name
        .parse::<api::types::CreateGroupMentionName>()
        .map_err(|e| format!("Invalid mention name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::CreateGroupDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let member_ids = resolve_members(&args.member_ids, client, cache_dir).await?;

    let group = client
        .create_group()
        .body_map(|mut b| {
            b = b.name(name).mention_name(mention_name);
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if !member_ids.is_empty() {
                b = b.member_ids(member_ids);
            }
            if !args.workflow_ids.is_empty() {
                b = b.workflow_ids(args.workflow_ids.clone());
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create group: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*group)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        out_println!(out, "{}", group.id);
        return Ok(());
    }

    out_println!(
        out,
        "Created group {} - {} (@{})",
        group.id,
        group.name,
        group.mention_name.as_str()
    );
    Ok(())
}
