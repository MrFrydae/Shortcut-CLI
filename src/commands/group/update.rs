use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;

use super::helpers::{resolve_group_id, resolve_members};

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// Group @mention_name or UUID
    #[arg(long)]
    pub id: String,

    /// The group name
    #[arg(long)]
    pub name: Option<String>,

    /// The @mention name for the group
    #[arg(long)]
    pub mention_name: Option<String>,

    /// The group description
    #[arg(long)]
    pub description: Option<String>,

    /// Whether the group is archived
    #[arg(long)]
    pub archived: Option<bool>,

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
    args: &UpdateArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let group_id = resolve_group_id(&args.id, client, cache_dir).await?;

    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateGroupName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let mention_name = args
        .mention_name
        .as_ref()
        .map(|m| m.parse::<api::types::UpdateGroupMentionName>())
        .transpose()
        .map_err(|e| format!("Invalid mention name: {e}"))?;

    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateGroupDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let member_ids = resolve_members(&args.member_ids, client, cache_dir).await?;

    let group = client
        .update_group()
        .group_public_id(group_id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(mention) = mention_name {
                b = b.mention_name(Some(mention));
            }
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(archived) = args.archived {
                b = b.archived(Some(archived));
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
        .map_err(|e| format!("Failed to update group: {e}"))?;

    println!("Updated group {} - {}", group.id, group.name);
    Ok(())
}
