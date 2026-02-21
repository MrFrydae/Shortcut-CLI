use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;
use crate::output::{OutputConfig, Table, format_template};

use super::super::member;
use super::helpers::resolve_workflow_state_id;
use crate::out_println;

#[derive(Args)]
pub struct ListArgs {
    /// Filter by owner (@mention_name or UUID)
    #[arg(long)]
    pub owner: Option<String>,

    /// Filter by workflow state name or ID
    #[arg(long)]
    pub state: Option<String>,

    /// Filter by epic ID
    #[arg(long)]
    pub epic_id: Option<i64>,

    /// Filter by story type (feature, bug, chore)
    #[arg(long, name = "type")]
    pub story_type: Option<String>,

    /// Filter by label name
    #[arg(long)]
    pub label: Option<String>,

    /// Filter by project ID
    #[arg(long)]
    pub project_id: Option<i64>,

    /// Maximum number of stories to display (default 25)
    #[arg(long, default_value = "25")]
    pub limit: i64,

    /// Include story descriptions in output
    #[arg(long, visible_alias = "descriptions")]
    pub desc: bool,
}

pub async fn run(
    args: &ListArgs,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let resolved_owner_id = match &args.owner {
        Some(val) => Some(member::resolve_member_id(val, client, cache_dir).await?),
        None => None,
    };

    let resolved_state_id = match &args.state {
        Some(val) => Some(resolve_workflow_state_id(val, client, cache_dir).await?),
        None => None,
    };

    let resolved_story_type = args
        .story_type
        .as_ref()
        .map(|t| t.parse::<api::types::SearchStoriesStoryType>())
        .transpose()
        .map_err(|e| format!("Invalid story type: {e}"))?;

    let resolved_label_name = args
        .label
        .as_ref()
        .map(|l| l.parse::<api::types::SearchStoriesLabelName>())
        .transpose()
        .map_err(|e| format!("Invalid label name: {e}"))?;

    let include_desc = args.desc;
    let epic_id = args.epic_id;
    let project_id = args.project_id;

    let stories = client
        .query_stories()
        .body_map(|mut b| {
            if let Some(owner_id) = resolved_owner_id {
                b = b.owner_id(Some(owner_id));
            }
            if let Some(state_id) = resolved_state_id {
                b = b.workflow_state_id(Some(state_id));
            }
            if let Some(epic_id) = epic_id {
                b = b.epic_id(Some(epic_id));
            }
            if let Some(st) = resolved_story_type {
                b = b.story_type(Some(st));
            }
            if let Some(label) = resolved_label_name {
                b = b.label_name(Some(label));
            }
            if let Some(pid) = project_id {
                b = b.project_id(Some(pid));
            }
            if include_desc {
                b = b.includes_description(Some(true));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to search stories: {e}"))?;

    let limit = args.limit as usize;
    let items: Vec<_> = stories.iter().take(limit).collect();

    if out.is_json() {
        let json: Vec<serde_json::Value> = items
            .iter()
            .map(|s| {
                serde_json::json!({
                    "id": s.id,
                    "name": s.name,
                    "story_type": s.story_type,
                    "workflow_state_id": s.workflow_state_id,
                })
            })
            .collect();
        out_println!(out, "{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    if out.is_quiet() {
        for story in &items {
            out_println!(out, "{}", story.id);
        }
        return Ok(());
    }

    if let Some(template) = out.format_template() {
        for story in &items {
            let val = serde_json::json!({
                "id": story.id,
                "name": story.name,
                "story_type": story.story_type,
                "workflow_state_id": story.workflow_state_id,
            });
            out_println!(out, "{}", format_template(template, &val)?);
        }
        return Ok(());
    }

    if items.is_empty() {
        out_println!(out, "No stories found");
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "Type", "State", "Name"]);
    for story in &items {
        table.add_row(vec![
            story.id.to_string(),
            story.story_type.to_string(),
            story.workflow_state_id.to_string(),
            story.name.clone(),
        ]);
    }
    out.write_str(format_args!("{}", table.render()))?;

    if args.desc {
        for story in &items {
            if let Some(d) = &story.description {
                out_println!(out, "  {}: {d}", story.id);
            }
        }
    }

    Ok(())
}
