use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;
use crate::output::OutputConfig;

use super::super::member;
use crate::out_println;

#[derive(Args)]
pub struct HistoryArgs {
    /// The ID of the story
    #[arg(long)]
    pub id: i64,
    /// Maximum number of history entries to display
    #[arg(long)]
    pub limit: Option<usize>,
}

pub async fn run(
    args: &HistoryArgs,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let entries = client
        .story_history()
        .story_public_id(args.id)
        .send()
        .await
        .map_err(|e| format!("Failed to get story history: {e}"))?;

    let mut entries: Vec<&api::types::History> = entries.iter().collect();
    entries.sort_by(|a, b| a.changed_at.cmp(&b.changed_at));

    if let Some(limit) = args.limit {
        let len = entries.len();
        if len > limit {
            entries = entries.split_off(len - limit);
        }
    }

    if entries.is_empty() {
        out_println!(out, "No history found for story {}", args.id);
        return Ok(());
    }

    // Build member lookup: uuid -> @mention_name
    let member_map = build_member_lookup(cache_dir, client).await?;

    // Build state and label lookups from references
    let mut state_map: HashMap<i64, String> = HashMap::new();
    let mut label_map: HashMap<i64, String> = HashMap::new();
    for entry in &entries {
        for reference in &entry.references {
            match reference {
                api::types::HistoryReferencesItem::WorkflowState(ws) => {
                    if let api::types::HistoryReferenceWorkflowStateId::Variant0(id) = ws.id {
                        state_map.insert(id, ws.name.clone());
                    }
                }
                api::types::HistoryReferencesItem::Label(lb) => {
                    if let api::types::HistoryReferenceLabelId::Variant0(id) = lb.id {
                        label_map.insert(id, lb.name.clone());
                    }
                }
                _ => {}
            }
        }
    }

    for entry in &entries {
        let who = entry
            .member_id
            .as_ref()
            .and_then(|id| member_map.get(&id.to_string()))
            .map(|m| format!("@{m}"))
            .unwrap_or_else(|| "(system)".to_string());

        for action in &entry.actions {
            if let Some(line) = format_action(action, &state_map, &label_map, &member_map) {
                out_println!(out, "[{}] {who}: {line}", entry.changed_at);
            }
        }
    }

    Ok(())
}

async fn build_member_lookup(
    cache_dir: &Path,
    client: &api::Client,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
    // Try cache first: mention_name -> uuid, we need uuid -> mention_name
    if let Some(cache) = member::read_cache(cache_dir) {
        let reverse: HashMap<String, String> = cache
            .into_iter()
            .map(|(mention, uuid)| (uuid, mention))
            .collect();
        if !reverse.is_empty() {
            return Ok(reverse);
        }
    }

    // Cache miss — fetch members
    let members = client
        .list_members()
        .send()
        .await
        .map_err(|e| format!("Failed to list members: {e}"))?;

    let map: HashMap<String, String> = members
        .iter()
        .map(|m| (m.id.to_string(), m.profile.mention_name.clone()))
        .collect();

    Ok(map)
}

fn format_action(
    action: &api::types::HistoryActionsItem,
    state_map: &HashMap<i64, String>,
    label_map: &HashMap<i64, String>,
    member_map: &HashMap<String, String>,
) -> Option<String> {
    use api::types::HistoryActionsItem::*;
    match action {
        StoryCreate(a) => Some(format!(
            "created story {} \"{}\" ({})",
            a.id, a.name, a.story_type
        )),
        StoryUpdate(a) => {
            let changes = format_story_update(a.changes.as_ref(), state_map, label_map, member_map);
            if changes.is_empty() {
                Some(format!("updated story {} \"{}\"", a.id, a.name))
            } else {
                Some(format!(
                    "updated story {} \"{}\": {}",
                    a.id, a.name, changes
                ))
            }
        }
        StoryDelete(a) => Some(format!(
            "deleted story {} \"{}\" ({})",
            a.id, a.name, a.story_type
        )),
        TaskCreate(a) => Some(format!("added task {} \"{}\"", a.id, a.description)),
        TaskUpdate(a) => Some(format!("updated task {} \"{}\"", a.id, a.description)),
        TaskDelete(a) => Some(format!("deleted task {} \"{}\"", a.id, a.description)),
        StoryCommentCreate(a) => Some(format!("added comment {}", a.id)),
        StoryLinkCreate(a) => Some(format!(
            "created link {} ({} {} -> {})",
            a.id, a.verb, a.subject_id, a.object_id
        )),
        StoryLinkUpdate(a) => Some(format!(
            "updated link {} ({} {} -> {})",
            a.id, a.verb, a.subject_id, a.object_id
        )),
        StoryLinkDelete(a) => Some(format!("deleted link {}", a.id)),
        LabelCreate(a) => Some(format!("added label \"{}\"", a.name)),
        LabelUpdate(a) => Some(format!("updated label {}", a.id)),
        LabelDelete(a) => Some(format!("removed label \"{}\"", a.name)),
        BranchCreate(a) => Some(format!("created branch \"{}\"", a.name)),
        BranchMerge(a) => Some(format!("merged branch \"{}\"", a.name)),
        BranchPush(a) => Some(format!("pushed to branch \"{}\"", a.name)),
        PullRequest(a) => Some(format!("pull request #{} \"{}\"", a.number, a.title)),
        ProjectUpdate(a) => Some(format!("updated project \"{}\"", a.name)),
        Workspace2BulkUpdate(a) => Some(format!("bulk update \"{}\"", a.name)),
    }
}

fn format_story_update(
    changes: Option<&api::types::HistoryChangesStory>,
    state_map: &HashMap<i64, String>,
    label_map: &HashMap<i64, String>,
    member_map: &HashMap<String, String>,
) -> String {
    let Some(c) = changes else {
        return String::new();
    };

    let mut parts: Vec<String> = Vec::new();

    if let Some(ws) = &c.workflow_state_id {
        let old_name = ws
            .old
            .and_then(|id| state_map.get(&id))
            .map(|s| s.as_str())
            .unwrap_or("(none)");
        let new_name = ws
            .new
            .and_then(|id| state_map.get(&id))
            .map(|s| s.as_str())
            .unwrap_or("(none)");
        parts.push(format!("state: {old_name} -> {new_name}"));
    }

    if let Some(n) = &c.name {
        let old = n.old.as_deref().unwrap_or("(none)");
        let new = n.new.as_deref().unwrap_or("(none)");
        parts.push(format!("name: \"{old}\" -> \"{new}\""));
    }

    if let Some(st) = &c.story_type {
        let old = st.old.as_deref().unwrap_or("(none)");
        let new = st.new.as_deref().unwrap_or("(none)");
        parts.push(format!("type: {old} -> {new}"));
    }

    if let Some(e) = &c.estimate {
        let old = e
            .old
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        let new = e
            .new
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        parts.push(format!("estimate: {old} -> {new}"));
    }

    if let Some(e) = &c.epic_id {
        let old = e
            .old
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        let new = e
            .new
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        parts.push(format!("epic_id: {old} -> {new}"));
    }

    if let Some(i) = &c.iteration_id {
        let old = i
            .old
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        let new = i
            .new
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        parts.push(format!("iteration_id: {old} -> {new}"));
    }

    if let Some(d) = &c.deadline {
        let old = d.old.as_deref().unwrap_or("(none)");
        let new = d.new.as_deref().unwrap_or("(none)");
        parts.push(format!("deadline: {old} -> {new}"));
    }

    if c.description.is_some() {
        parts.push("description: (changed)".to_string());
    }

    if let Some(l) = &c.label_ids {
        if !l.adds.is_empty() {
            let names: Vec<&str> = l
                .adds
                .iter()
                .map(|id| label_map.get(id).map(|s| s.as_str()).unwrap_or("unknown"))
                .collect();
            parts.push(format!("labels added: {}", names.join(", ")));
        }
        if !l.removes.is_empty() {
            let names: Vec<&str> = l
                .removes
                .iter()
                .map(|id| label_map.get(id).map(|s| s.as_str()).unwrap_or("unknown"))
                .collect();
            parts.push(format!("labels removed: {}", names.join(", ")));
        }
    }

    if let Some(o) = &c.owner_ids {
        if !o.adds.is_empty() {
            let names: Vec<String> = o
                .adds
                .iter()
                .map(|id| {
                    member_map
                        .get(&id.to_string())
                        .map(|m| format!("@{m}"))
                        .unwrap_or_else(|| id.to_string())
                })
                .collect();
            parts.push(format!("owners added: {}", names.join(", ")));
        }
        if !o.removes.is_empty() {
            let names: Vec<String> = o
                .removes
                .iter()
                .map(|id| {
                    member_map
                        .get(&id.to_string())
                        .map(|m| format!("@{m}"))
                        .unwrap_or_else(|| id.to_string())
                })
                .collect();
            parts.push(format!("owners removed: {}", names.join(", ")));
        }
    }

    if let Some(s) = &c.started {
        parts.push(format!(
            "started: {} -> {}",
            s.old.unwrap_or(false),
            s.new.unwrap_or(false)
        ));
    }

    if let Some(s) = &c.completed {
        parts.push(format!(
            "completed: {} -> {}",
            s.old.unwrap_or(false),
            s.new.unwrap_or(false)
        ));
    }

    if let Some(s) = &c.archived {
        parts.push(format!(
            "archived: {} -> {}",
            s.old.unwrap_or(false),
            s.new.unwrap_or(false)
        ));
    }

    if let Some(s) = &c.blocked {
        parts.push(format!(
            "blocked: {} -> {}",
            s.old.unwrap_or(false),
            s.new.unwrap_or(false)
        ));
    }

    if let Some(s) = &c.blocker {
        parts.push(format!(
            "blocker: {} -> {}",
            s.old.unwrap_or(false),
            s.new.unwrap_or(false)
        ));
    }

    if let Some(p) = &c.project_id {
        let old = p
            .old
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        let new = p
            .new
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        parts.push(format!("project_id: {old} -> {new}"));
    }

    if let Some(g) = &c.group_id {
        let old = g
            .old
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        let new = g
            .new
            .map(|v| v.to_string())
            .unwrap_or_else(|| "(none)".to_string());
        parts.push(format!("group_id: {old} -> {new}"));
    }

    parts.join(", ")
}
