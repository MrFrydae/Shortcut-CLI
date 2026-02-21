use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;

use super::super::member;
use super::helpers::resolve_workflow_state_id;

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
    let mut count = 0;

    for story in stories.iter() {
        if count >= limit {
            break;
        }
        println!(
            "{} - {} ({}, state_id: {})",
            story.id, story.name, story.story_type, story.workflow_state_id
        );
        if args.desc
            && let Some(d) = &story.description
        {
            println!("  {d}");
        }
        count += 1;
    }

    if count == 0 {
        println!("No stories found");
    }

    Ok(())
}
