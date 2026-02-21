use std::error::Error;
use std::path::Path;

use crate::api;
use crate::output::OutputConfig;

use super::super::custom_field;
use super::link::invert_verb;
use crate::out_println;

pub async fn run(
    id: i64,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let story = client
        .get_story()
        .story_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get story: {e}"))?;

    if out.is_json() {
        let json = serde_json::json!({
            "id": story.id,
            "name": story.name,
            "story_type": story.story_type,
            "workflow_state_id": story.workflow_state_id,
            "workflow_id": story.workflow_id,
            "epic_id": story.epic_id,
            "estimate": story.estimate,
            "owner_ids": story.owner_ids,
            "labels": story.labels.iter().map(|l| &l.name).collect::<Vec<_>>(),
            "description": story.description,
            "branches": story.branches.iter().map(|b| serde_json::json!({
                "name": b.name,
                "url": b.url,
                "repository_id": b.repository_id,
            })).collect::<Vec<_>>(),
            "pull_requests": story.pull_requests.iter().map(|pr| {
                serde_json::json!({
                    "number": pr.number,
                    "title": pr.title,
                    "url": pr.url,
                    "status": pr_status(pr),
                    "repository_id": pr.repository_id,
                })
            }).collect::<Vec<_>>(),
        });
        out_println!(out, "{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    if out.is_quiet() {
        out_println!(out, "{}", story.id);
        return Ok(());
    }

    out_println!(out, "{} - {}", story.id, story.name);
    out_println!(out, "  Type:        {}", story.story_type);
    out_println!(out, "  State ID:    {}", story.workflow_state_id);
    out_println!(out, "  Workflow ID: {}", story.workflow_id);
    if let Some(epic_id) = story.epic_id {
        out_println!(out, "  Epic ID:     {epic_id}");
    }
    if let Some(estimate) = story.estimate {
        out_println!(out, "  Estimate:    {estimate}");
    }
    if !story.owner_ids.is_empty() {
        let ids: Vec<String> = story.owner_ids.iter().map(|id| id.to_string()).collect();
        out_println!(out, "  Owners:      {}", ids.join(", "));
    }
    if !story.labels.is_empty() {
        let names: Vec<&str> = story.labels.iter().map(|l| l.name.as_str()).collect();
        out_println!(out, "  Labels:      {}", names.join(", "));
    }
    if !story.description.is_empty() {
        out_println!(out, "  Description: {}", story.description);
    }
    if !story.story_links.is_empty() {
        out_println!(out, "  Links:");
        for link in &story.story_links {
            let (display_verb, other_id) = if link.type_ == "subject" {
                (link.verb.as_str(), link.object_id)
            } else {
                (invert_verb(&link.verb), link.subject_id)
            };
            out_println!(out, "    {display_verb} {other_id} (link {})", link.id);
        }
    }
    if !story.custom_fields.is_empty() {
        let field_ids: Vec<uuid::Uuid> = story.custom_fields.iter().map(|cf| cf.field_id).collect();
        let names = custom_field::resolve_custom_field_names(&field_ids, client, cache_dir).await?;
        for cf in &story.custom_fields {
            let field_name = names
                .get(&cf.field_id.to_string())
                .map(|s| s.as_str())
                .unwrap_or("Unknown");
            out_println!(out, "  {}: {}", field_name, cf.value);
        }
    }
    if !story.branches.is_empty() {
        out_println!(out, "  Branches:");
        for branch in &story.branches {
            out_println!(out, "    {} (repo {})", branch.name, branch.repository_id);
        }
    }
    if !story.pull_requests.is_empty() {
        out_println!(out, "  Pull Requests:");
        for pr in &story.pull_requests {
            let status = pr_status(pr);
            out_println!(
                out,
                "    #{} \"{}\" ({}) — {}",
                pr.number,
                pr.title,
                status,
                pr.url
            );
        }
    }

    Ok(())
}

fn pr_status(pr: &crate::api::types::PullRequest) -> &'static str {
    if pr.merged {
        "merged"
    } else if pr.closed {
        "closed"
    } else if pr.draft {
        "draft"
    } else {
        "open"
    }
}
