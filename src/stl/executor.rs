use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use progenitor_client::ClientInfo;

use crate::api;
use crate::commands::{custom_field, epic, group, member, story};
use crate::out_println;
use crate::output::OutputConfig;

use super::resolver::{resolve_refs, substitute_vars, yaml_mapping_to_json, yaml_to_json};
use super::types::*;

/// Execute a validated template.
pub async fn execute(
    template: &mut Template,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
    confirm: bool,
) -> Result<ExecutionResult, Box<dyn Error>> {
    // Pre-pass: substitute all $var() in the entire operations tree
    let vars = template.vars.clone().unwrap_or_default();
    for op in &mut template.operations {
        if let Some(fields) = &mut op.fields {
            let mut val = serde_yaml::Value::Mapping(fields.clone());
            substitute_vars(&mut val, &vars).map_err(|errs| errs.join("; "))?;
            if let serde_yaml::Value::Mapping(m) = val {
                *fields = m;
            }
        }
        if let Some(repeat) = &mut op.repeat {
            for entry in repeat.iter_mut() {
                let mut val = serde_yaml::Value::Mapping(entry.clone());
                substitute_vars(&mut val, &vars).map_err(|errs| errs.join("; "))?;
                if let serde_yaml::Value::Mapping(m) = val {
                    *entry = m;
                }
            }
        }
        if let Some(id) = &mut op.id {
            substitute_vars(id, &vars).map_err(|errs| errs.join("; "))?;
        }
    }

    // Count total operations (expanding repeats)
    let total = count_operations(&template.operations);
    let doc_on_error = &template.on_error;
    let show_progress = !out.is_json();

    if !confirm && !out.is_dry_run() && show_progress {
        print_confirmation_summary(template, total, out)?;
        if !prompt_confirm()? {
            return Err("Aborted by user.".into());
        }
    }

    let mut results: HashMap<String, serde_json::Value> = HashMap::new();
    let mut op_results: Vec<OperationResult> = Vec::new();
    let mut op_counter = 0;

    let base_url = extract_base_url(client);
    let http = build_http_client(client)?;

    for op in template.operations.iter() {
        let should_continue_on_error = op
            .on_error
            .as_ref()
            .or(doc_on_error.as_ref())
            .map(|e| *e == ErrorHandling::Continue)
            .unwrap_or(false);

        if let Some(repeat) = &op.repeat {
            // Expand repeat into N sub-operations
            let shared_fields = op.fields.as_ref().cloned().unwrap_or_default();
            let mut repeat_results: Vec<serde_json::Value> = Vec::new();

            for entry in repeat.iter() {
                op_counter += 1;
                // Merge shared fields with repeat entry (entry overrides shared)
                let merged = merge_mappings(&shared_fields, entry);
                let mut json_body = yaml_mapping_to_json(&merged);

                // Resolve $ref()
                if let Err(e) = resolve_refs(&mut json_body, &results) {
                    let op_result = OperationResult {
                        index: op_counter - 1,
                        action: op.action.to_string(),
                        entity: op.entity.to_string(),
                        status: "failed".to_string(),
                        result: None,
                        error: Some(e.clone()),
                    };
                    if show_progress {
                        out_println!(
                            out,
                            "[{}/{}] FAILED: {} {} — {}",
                            op_counter,
                            total,
                            op.action,
                            op.entity,
                            e
                        );
                    }
                    op_results.push(op_result);
                    if !should_continue_on_error {
                        return Ok(build_result(op_results, total));
                    }
                    continue;
                }

                // Build URL before resolve_entity_fields, which may remove
                // fields (e.g. story_id) that are needed for URL construction.
                let (method, path) = resolve_api_path(&op.action, &op.entity, None, &json_body)?;

                // Resolve entity-specific fields
                if let Err(e) =
                    resolve_entity_fields(&op.entity, &op.action, &mut json_body, client, cache_dir)
                        .await
                {
                    let op_result = OperationResult {
                        index: op_counter - 1,
                        action: op.action.to_string(),
                        entity: op.entity.to_string(),
                        status: "failed".to_string(),
                        result: None,
                        error: Some(e.to_string()),
                    };
                    if show_progress {
                        out_println!(
                            out,
                            "[{}/{}] FAILED: {} {} — {}",
                            op_counter,
                            total,
                            op.action,
                            op.entity,
                            e
                        );
                    }
                    op_results.push(op_result);
                    if !should_continue_on_error {
                        return Ok(build_result(op_results, total));
                    }
                    continue;
                }

                if out.is_dry_run() {
                    if show_progress {
                        out_println!(out, "[{}/{}] {} {}", op_counter, total, method, path);
                        let pretty = serde_json::to_string_pretty(&json_body)?;
                        out_println!(out, "  {}", pretty.replace('\n', "\n  "));
                        out_println!(out, "");
                    }
                    let op_result = OperationResult {
                        index: op_counter - 1,
                        action: op.action.to_string(),
                        entity: op.entity.to_string(),
                        status: "success".to_string(),
                        result: Some(dry_run_placeholder(&op.entity, op_counter)),
                        error: None,
                    };
                    repeat_results.push(dry_run_placeholder(&op.entity, op_counter));
                    op_results.push(op_result);
                    continue;
                }

                // Execute the API call
                match execute_request(&http, &base_url, &method, &path, &json_body).await {
                    Ok(response) => {
                        if show_progress {
                            let name = response.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let id_display = response
                                .get("id")
                                .map(json_value_display)
                                .unwrap_or_default();
                            let action_past = action_past_tense(&op.action);
                            if name.is_empty() {
                                out_println!(
                                    out,
                                    "[{}/{}] {} {} {}",
                                    op_counter,
                                    total,
                                    action_past,
                                    op.entity,
                                    id_display
                                );
                            } else {
                                out_println!(
                                    out,
                                    "[{}/{}] {} {} {} - {}",
                                    op_counter,
                                    total,
                                    action_past,
                                    op.entity,
                                    id_display,
                                    name
                                );
                            }
                        }
                        repeat_results.push(response.clone());
                        op_results.push(OperationResult {
                            index: op_counter - 1,
                            action: op.action.to_string(),
                            entity: op.entity.to_string(),
                            status: "success".to_string(),
                            result: Some(response),
                            error: None,
                        });
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        if show_progress {
                            out_println!(
                                out,
                                "[{}/{}] FAILED: {} {} — {}",
                                op_counter,
                                total,
                                op.action,
                                op.entity,
                                err_msg
                            );
                        }
                        op_results.push(OperationResult {
                            index: op_counter - 1,
                            action: op.action.to_string(),
                            entity: op.entity.to_string(),
                            status: "failed".to_string(),
                            result: None,
                            error: Some(err_msg),
                        });
                        if !should_continue_on_error {
                            return Ok(build_result(op_results, total));
                        }
                    }
                }
            }

            // Store repeat results under alias (as array)
            if let Some(alias) = &op.alias {
                results.insert(alias.clone(), serde_json::Value::Array(repeat_results));
            }
        } else {
            // Single operation
            op_counter += 1;

            let mut json_body = op
                .fields
                .as_ref()
                .map(yaml_mapping_to_json)
                .unwrap_or(serde_json::json!({}));

            // Resolve the `id` field
            let resolved_id = if let Some(id_val) = &op.id {
                let mut json_id = yaml_to_json(id_val);
                resolve_refs(&mut json_id, &results).map_err(|e| e.to_string())?;
                Some(json_id)
            } else {
                None
            };

            // Resolve $ref() in body
            if let Err(e) = resolve_refs(&mut json_body, &results) {
                let op_result = OperationResult {
                    index: op_counter - 1,
                    action: op.action.to_string(),
                    entity: op.entity.to_string(),
                    status: "failed".to_string(),
                    result: None,
                    error: Some(e.clone()),
                };
                if show_progress {
                    out_println!(
                        out,
                        "[{}/{}] FAILED: {} {} — {}",
                        op_counter,
                        total,
                        op.action,
                        op.entity,
                        e
                    );
                }
                op_results.push(op_result);
                if !should_continue_on_error {
                    return Ok(build_result(op_results, total));
                }
                continue;
            }

            // Build URL before resolve_entity_fields, which may remove
            // fields (e.g. story_id) that are needed for URL construction.
            let (method, path) =
                resolve_api_path(&op.action, &op.entity, resolved_id.as_ref(), &json_body)?;

            // Resolve entity-specific fields
            if let Err(e) =
                resolve_entity_fields(&op.entity, &op.action, &mut json_body, client, cache_dir)
                    .await
            {
                let op_result = OperationResult {
                    index: op_counter - 1,
                    action: op.action.to_string(),
                    entity: op.entity.to_string(),
                    status: "failed".to_string(),
                    result: None,
                    error: Some(e.to_string()),
                };
                if show_progress {
                    out_println!(
                        out,
                        "[{}/{}] FAILED: {} {} — {}",
                        op_counter,
                        total,
                        op.action,
                        op.entity,
                        e
                    );
                }
                op_results.push(op_result);
                if !should_continue_on_error {
                    return Ok(build_result(op_results, total));
                }
                continue;
            }

            if out.is_dry_run() {
                if show_progress {
                    out_println!(out, "[{}/{}] {} {}", op_counter, total, method, path);
                    if !json_body_is_empty(&json_body) {
                        let pretty = serde_json::to_string_pretty(&json_body)?;
                        out_println!(out, "  {}", pretty.replace('\n', "\n  "));
                    }
                    out_println!(out, "");
                }
                let placeholder = dry_run_placeholder(&op.entity, op_counter);
                if let Some(alias) = &op.alias {
                    results.insert(alias.clone(), placeholder.clone());
                }
                op_results.push(OperationResult {
                    index: op_counter - 1,
                    action: op.action.to_string(),
                    entity: op.entity.to_string(),
                    status: "success".to_string(),
                    result: Some(placeholder),
                    error: None,
                });
                continue;
            }

            // Execute the API call
            match execute_request(&http, &base_url, &method, &path, &json_body).await {
                Ok(response) => {
                    if show_progress {
                        let action_past = action_past_tense(&op.action);
                        print_success_line(
                            out,
                            op_counter,
                            total,
                            &action_past,
                            &op.action,
                            &op.entity,
                            &response,
                            resolved_id.as_ref(),
                        )?;
                    }
                    if let Some(alias) = &op.alias {
                        results.insert(alias.clone(), response.clone());
                    }
                    op_results.push(OperationResult {
                        index: op_counter - 1,
                        action: op.action.to_string(),
                        entity: op.entity.to_string(),
                        status: "success".to_string(),
                        result: Some(response),
                        error: None,
                    });
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    if show_progress {
                        out_println!(
                            out,
                            "[{}/{}] FAILED: {} {} — {}",
                            op_counter,
                            total,
                            op.action,
                            op.entity,
                            err_msg
                        );
                    }
                    op_results.push(OperationResult {
                        index: op_counter - 1,
                        action: op.action.to_string(),
                        entity: op.entity.to_string(),
                        status: "failed".to_string(),
                        result: None,
                        error: Some(err_msg),
                    });
                    if !should_continue_on_error {
                        return Ok(build_result(op_results, total));
                    }
                }
            }
        }
    }

    Ok(build_result(op_results, total))
}

// ── Helpers ────────────────────────────────────────────────────────

/// Count total operations including repeat expansions.
fn count_operations(operations: &[Operation]) -> usize {
    operations
        .iter()
        .map(|op| op.repeat.as_ref().map(|r| r.len()).unwrap_or(1))
        .sum()
}

/// Print the pre-execution confirmation summary.
fn print_confirmation_summary(
    template: &Template,
    total: usize,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if let Some(meta) = &template.meta
        && let Some(desc) = &meta.description
    {
        out_println!(out, "Description: {desc}");
    }
    out_println!(out, "");

    // Summarize by action+entity
    let mut counts: HashMap<(String, String), usize> = HashMap::new();
    for op in &template.operations {
        let n = op.repeat.as_ref().map(|r| r.len()).unwrap_or(1);
        *counts
            .entry((op.action.to_string(), op.entity.to_string()))
            .or_default() += n;
    }

    out_println!(out, "Will execute {total} operations:");
    let mut entries: Vec<_> = counts.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    for ((action, entity), count) in entries {
        let plural = if count > 1 {
            format!("{entity}s")
        } else {
            entity.clone()
        };
        out_println!(out, "  {action:<8} {count} {plural}");
    }
    out_println!(out, "");

    Ok(())
}

/// Prompt user for y/N confirmation.
fn prompt_confirm() -> Result<bool, Box<dyn Error>> {
    use std::io::Write;
    eprint!("Proceed? [y/N] ");
    std::io::stderr().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();
    Ok(answer == "y" || answer == "yes")
}

/// Build the final execution result with summary.
fn build_result(op_results: Vec<OperationResult>, total: usize) -> ExecutionResult {
    let succeeded = op_results.iter().filter(|r| r.status == "success").count();
    let failed = op_results.iter().filter(|r| r.status == "failed").count();
    ExecutionResult {
        operations: op_results,
        summary: ExecutionSummary {
            total,
            succeeded,
            failed,
        },
    }
}

/// Merge two YAML mappings. Values from `override_mapping` take precedence.
fn merge_mappings(
    base: &serde_yaml::Mapping,
    override_mapping: &serde_yaml::Mapping,
) -> serde_yaml::Mapping {
    let mut merged = base.clone();
    for (k, v) in override_mapping {
        merged.insert(k.clone(), v.clone());
    }
    merged
}

/// Resolve the HTTP method and API path for an operation.
fn resolve_api_path(
    action: &Action,
    entity: &Entity,
    id: Option<&serde_json::Value>,
    body: &serde_json::Value,
) -> Result<(String, String), Box<dyn Error>> {
    let id_str = id.map(json_value_display);

    match (action, entity) {
        // Story
        (Action::Create, Entity::Story) => Ok(("POST".into(), "/api/v3/stories".into())),
        (Action::Update, Entity::Story) => {
            let id = id_str.ok_or("update story requires id")?;
            Ok(("PUT".into(), format!("/api/v3/stories/{id}")))
        }
        (Action::Delete, Entity::Story) => {
            let id = id_str.ok_or("delete story requires id")?;
            Ok(("DELETE".into(), format!("/api/v3/stories/{id}")))
        }

        // Epic
        (Action::Create, Entity::Epic) => Ok(("POST".into(), "/api/v3/epics".into())),
        (Action::Update, Entity::Epic) => {
            let id = id_str.ok_or("update epic requires id")?;
            Ok(("PUT".into(), format!("/api/v3/epics/{id}")))
        }
        (Action::Delete, Entity::Epic) => {
            let id = id_str.ok_or("delete epic requires id")?;
            Ok(("DELETE".into(), format!("/api/v3/epics/{id}")))
        }

        // Iteration
        (Action::Create, Entity::Iteration) => Ok(("POST".into(), "/api/v3/iterations".into())),
        (Action::Update, Entity::Iteration) => {
            let id = id_str.ok_or("update iteration requires id")?;
            Ok(("PUT".into(), format!("/api/v3/iterations/{id}")))
        }
        (Action::Delete, Entity::Iteration) => {
            let id = id_str.ok_or("delete iteration requires id")?;
            Ok(("DELETE".into(), format!("/api/v3/iterations/{id}")))
        }

        // Label
        (Action::Create, Entity::Label) => Ok(("POST".into(), "/api/v3/labels".into())),
        (Action::Update, Entity::Label) => {
            let id = id_str.ok_or("update label requires id")?;
            Ok(("PUT".into(), format!("/api/v3/labels/{id}")))
        }
        (Action::Delete, Entity::Label) => {
            let id = id_str.ok_or("delete label requires id")?;
            Ok(("DELETE".into(), format!("/api/v3/labels/{id}")))
        }

        // Objective
        (Action::Create, Entity::Objective) => Ok(("POST".into(), "/api/v3/objectives".into())),
        (Action::Update, Entity::Objective) => {
            let id = id_str.ok_or("update objective requires id")?;
            Ok(("PUT".into(), format!("/api/v3/objectives/{id}")))
        }
        (Action::Delete, Entity::Objective) => {
            let id = id_str.ok_or("delete objective requires id")?;
            Ok(("DELETE".into(), format!("/api/v3/objectives/{id}")))
        }

        // Milestone
        (Action::Create, Entity::Milestone) => Ok(("POST".into(), "/api/v3/milestones".into())),
        (Action::Update, Entity::Milestone) => {
            let id = id_str.ok_or("update milestone requires id")?;
            Ok(("PUT".into(), format!("/api/v3/milestones/{id}")))
        }
        (Action::Delete, Entity::Milestone) => {
            let id = id_str.ok_or("delete milestone requires id")?;
            Ok(("DELETE".into(), format!("/api/v3/milestones/{id}")))
        }

        // Category
        (Action::Create, Entity::Category) => Ok(("POST".into(), "/api/v3/categories".into())),
        (Action::Update, Entity::Category) => {
            let id = id_str.ok_or("update category requires id")?;
            Ok(("PUT".into(), format!("/api/v3/categories/{id}")))
        }
        (Action::Delete, Entity::Category) => {
            let id = id_str.ok_or("delete category requires id")?;
            Ok(("DELETE".into(), format!("/api/v3/categories/{id}")))
        }

        // Group
        (Action::Create, Entity::Group) => Ok(("POST".into(), "/api/v3/groups".into())),
        (Action::Update, Entity::Group) => {
            let id = id_str.ok_or("update group requires id")?;
            Ok(("PUT".into(), format!("/api/v3/groups/{id}")))
        }

        // Document
        (Action::Create, Entity::Document) => Ok(("POST".into(), "/api/v3/documents".into())),
        (Action::Update, Entity::Document) => {
            let id = id_str.ok_or("update document requires id")?;
            Ok(("PUT".into(), format!("/api/v3/documents/{id}")))
        }
        (Action::Delete, Entity::Document) => {
            let id = id_str.ok_or("delete document requires id")?;
            Ok(("DELETE".into(), format!("/api/v3/documents/{id}")))
        }

        // Project
        (Action::Create, Entity::Project) => Ok(("POST".into(), "/api/v3/projects".into())),
        (Action::Update, Entity::Project) => {
            let id = id_str.ok_or("update project requires id")?;
            Ok(("PUT".into(), format!("/api/v3/projects/{id}")))
        }
        (Action::Delete, Entity::Project) => {
            let id = id_str.ok_or("delete project requires id")?;
            Ok(("DELETE".into(), format!("/api/v3/projects/{id}")))
        }

        // Task — requires story_id from body
        (Action::Create, Entity::Task) => {
            let story_id = body
                .get("story_id")
                .map(json_value_display)
                .ok_or("create task requires 'story_id' in fields")?;
            Ok(("POST".into(), format!("/api/v3/stories/{story_id}/tasks")))
        }

        // Comment on story or epic
        (Action::Comment, Entity::Story) => {
            let id = id_str.ok_or("comment on story requires id")?;
            Ok(("POST".into(), format!("/api/v3/stories/{id}/comments")))
        }
        (Action::Comment, Entity::Epic) => {
            let id = id_str.ok_or("comment on epic requires id")?;
            Ok(("POST".into(), format!("/api/v3/epics/{id}/comments")))
        }

        // Story Link
        (Action::Link, Entity::StoryLink) => Ok(("POST".into(), "/api/v3/story-links".into())),
        (Action::Unlink, Entity::StoryLink) => {
            let id = id_str.ok_or("unlink story-link requires id")?;
            Ok(("DELETE".into(), format!("/api/v3/story-links/{id}")))
        }

        // Check/Uncheck task — requires story_id and task_id
        (Action::Check, Entity::Task) => {
            let task_id = id_str.ok_or("check task requires id")?;
            let story_id = body
                .get("story_id")
                .map(json_value_display)
                .ok_or("check task requires 'story_id' in fields")?;
            Ok((
                "PUT".into(),
                format!("/api/v3/stories/{story_id}/tasks/{task_id}"),
            ))
        }
        (Action::Uncheck, Entity::Task) => {
            let task_id = id_str.ok_or("uncheck task requires id")?;
            let story_id = body
                .get("story_id")
                .map(json_value_display)
                .ok_or("uncheck task requires 'story_id' in fields")?;
            Ok((
                "PUT".into(),
                format!("/api/v3/stories/{story_id}/tasks/{task_id}"),
            ))
        }

        _ => Err(format!(
            "unsupported action/entity combination: {} {}",
            action, entity
        )
        .into()),
    }
}

/// Resolve entity-specific fields (members, states, etc.) in the JSON body.
async fn resolve_entity_fields(
    entity: &Entity,
    action: &Action,
    body: &mut serde_json::Value,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let obj = match body.as_object_mut() {
        Some(o) => o,
        None => return Ok(()),
    };

    // Resolve owner → owner_ids (single member)
    if let Some(owner_val) = obj.remove("owner")
        && let Some(mention) = owner_val.as_str()
    {
        let uuid = member::resolve_member_id(mention, client, cache_dir).await?;
        obj.insert("owner_ids".into(), serde_json::json!([uuid.to_string()]));
    }

    // Resolve owners → owner_ids (multiple members)
    if let Some(owners_val) = obj.remove("owners")
        && let Some(arr) = owners_val.as_array()
    {
        let mut ids = Vec::new();
        for item in arr {
            if let Some(mention) = item.as_str() {
                let uuid = member::resolve_member_id(mention, client, cache_dir).await?;
                ids.push(serde_json::Value::String(uuid.to_string()));
            }
        }
        obj.insert("owner_ids".into(), serde_json::Value::Array(ids));
    }

    // Resolve followers → follower_ids
    if let Some(followers_val) = obj.remove("followers")
        && let Some(arr) = followers_val.as_array()
    {
        let mut ids = Vec::new();
        for item in arr {
            if let Some(mention) = item.as_str() {
                let uuid = member::resolve_member_id(mention, client, cache_dir).await?;
                ids.push(serde_json::Value::String(uuid.to_string()));
            }
        }
        obj.insert("follower_ids".into(), serde_json::Value::Array(ids));
    }

    // Resolve requested_by → requested_by_id
    if let Some(rb_val) = obj.remove("requested_by")
        && let Some(mention) = rb_val.as_str()
    {
        let uuid = member::resolve_member_id(mention, client, cache_dir).await?;
        obj.insert(
            "requested_by_id".into(),
            serde_json::Value::String(uuid.to_string()),
        );
    }

    // Resolve member_ids (group members)
    if let Some(member_ids_val) = obj.remove("member_ids")
        && let Some(arr) = member_ids_val.as_array()
    {
        let mut ids = Vec::new();
        for item in arr {
            if let Some(mention) = item.as_str() {
                let uuid = member::resolve_member_id(mention, client, cache_dir).await?;
                ids.push(serde_json::Value::String(uuid.to_string()));
            }
        }
        obj.insert("member_ids".into(), serde_json::Value::Array(ids));
    }

    // Resolve story state → workflow_state_id
    if *entity == Entity::Story
        && let Some(state_val) = obj.remove("state")
    {
        if let Some(state_str) = state_val.as_str() {
            let state_id =
                story::helpers::resolve_workflow_state_id(state_str, client, cache_dir).await?;
            obj.insert("workflow_state_id".into(), serde_json::json!(state_id));
        } else if let Some(state_id) = state_val.as_i64() {
            obj.insert("workflow_state_id".into(), serde_json::json!(state_id));
        }
    }

    // Resolve epic state → epic_state_id (mapped to "state" in the API)
    if *entity == Entity::Epic
        && let Some(state_val) = obj.remove("state")
    {
        if let Some(state_str) = state_val.as_str() {
            let state_id =
                epic::helpers::resolve_epic_state_id(state_str, client, cache_dir).await?;
            obj.insert("epic_state_id".into(), serde_json::json!(state_id));
        } else if let Some(state_id) = state_val.as_i64() {
            obj.insert("epic_state_id".into(), serde_json::json!(state_id));
        }
    }

    // Resolve story type → story_type
    if *entity == Entity::Story
        && let Some(type_val) = obj.remove("type")
    {
        obj.insert("story_type".into(), type_val);
    }

    // Resolve labels → [{"name": "label_name"}]
    if let Some(labels_val) = obj.remove("labels")
        && let Some(arr) = labels_val.as_array()
    {
        let label_objects: Vec<serde_json::Value> = arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| serde_json::json!({"name": s})))
            .collect();
        obj.insert("labels".into(), serde_json::Value::Array(label_objects));
    }

    // Resolve group_id → resolve_group_id
    if let Some(gid_val) = obj.remove("group_id") {
        if let Some(gid_str) = gid_val.as_str() {
            if gid_str.starts_with('@') {
                let uuid = group::helpers::resolve_group_id(gid_str, client, cache_dir).await?;
                obj.insert(
                    "group_id".into(),
                    serde_json::Value::String(uuid.to_string()),
                );
            } else {
                obj.insert("group_id".into(), gid_val);
            }
        } else {
            obj.insert("group_id".into(), gid_val);
        }
    }

    // Resolve group_ids
    if let Some(gids_val) = obj.remove("group_ids")
        && let Some(arr) = gids_val.as_array()
    {
        let mut ids = Vec::new();
        for item in arr {
            if let Some(gid_str) = item.as_str() {
                if gid_str.starts_with('@') {
                    let uuid = group::helpers::resolve_group_id(gid_str, client, cache_dir).await?;
                    ids.push(serde_json::Value::String(uuid.to_string()));
                } else {
                    ids.push(item.clone());
                }
            } else {
                ids.push(item.clone());
            }
        }
        obj.insert("group_ids".into(), serde_json::Value::Array(ids));
    }

    // Resolve custom_fields → custom field values
    if let Some(cf_val) = obj.remove("custom_fields")
        && let Some(cf_obj) = cf_val.as_object()
    {
        let mut cf_params = Vec::new();
        for (field_name, value_val) in cf_obj {
            if let Some(value_name) = value_val.as_str() {
                let (field_id, value_id) = custom_field::helpers::resolve_custom_field_value(
                    field_name, value_name, client, cache_dir,
                )
                .await?;
                cf_params.push(serde_json::json!({
                    "field_id": field_id.to_string(),
                    "value": null,
                    "value_id": value_id.to_string()
                }));
            }
        }
        obj.insert("custom_fields".into(), serde_json::Value::Array(cf_params));
    }

    // Resolve content_file → content (read file)
    if let Some(cf_val) = obj.remove("content_file")
        && let Some(path) = cf_val.as_str()
    {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read content_file '{path}': {e}"))?;
        obj.insert("content".into(), serde_json::Value::String(content));
    }

    // Resolve text_file → text (read file)
    if let Some(tf_val) = obj.remove("text_file")
        && let Some(path) = tf_val.as_str()
    {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read text_file '{path}': {e}"))?;
        obj.insert("text".into(), serde_json::Value::String(text));
    }

    // Resolve story_link verb normalization
    if *entity == Entity::StoryLink
        && action == &Action::Link
        && let Some(verb_val) = obj.remove("verb")
        && let Some(verb) = verb_val.as_str()
    {
        let normalized = normalize_link_verb(verb);
        obj.insert("verb".into(), serde_json::Value::String(normalized));
    }

    // For check/uncheck actions, set the complete flag
    if *entity == Entity::Task {
        match action {
            Action::Check => {
                obj.insert("complete".into(), serde_json::json!(true));
            }
            Action::Uncheck => {
                obj.insert("complete".into(), serde_json::json!(false));
            }
            _ => {}
        }
        // Remove story_id from the body since it's used in the URL
        obj.remove("story_id");
    }

    // For comment actions, remove story_id and epic_id from body (used in URL)
    if action == &Action::Comment {
        obj.remove("story_id");
        obj.remove("epic_id");
    }

    Ok(())
}

/// Normalize story link verb aliases.
fn normalize_link_verb(verb: &str) -> String {
    match verb.to_lowercase().as_str() {
        "blocks" => "blocks".to_string(),
        "blocked-by" | "blocked_by" | "is blocked by" => "is blocked by".to_string(),
        "duplicates" => "duplicates".to_string(),
        "relates-to" | "relates_to" | "relates to" => "relates to".to_string(),
        other => other.to_string(),
    }
}

/// Execute an HTTP request and parse the JSON response.
async fn execute_request(
    http: &reqwest::Client,
    base_url: &str,
    method: &str,
    path: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let url = format!("{base_url}{path}");

    let request = match method {
        "POST" => http.post(&url).json(body),
        "PUT" => http.put(&url).json(body),
        "DELETE" => http.delete(&url),
        _ => return Err(format!("unsupported HTTP method: {method}").into()),
    };

    let response = request.send().await?;
    let status = response.status();

    if status.is_success() {
        // DELETE may return empty body
        let text = response.text().await?;
        if text.is_empty() {
            Ok(serde_json::json!({}))
        } else {
            serde_json::from_str(&text)
                .map_err(|e| format!("Failed to parse API response: {e}").into())
        }
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("{status}: {error_text}").into())
    }
}

/// Build an HTTP client from the API client's inner reqwest client.
fn build_http_client(client: &api::Client) -> Result<reqwest::Client, Box<dyn Error>> {
    // We need a reqwest client with the Shortcut-Token header.
    // The api::Client wraps one, but we can't easily extract it.
    // Instead, we'll read the token from the client's base URL pattern
    // and build a new client. For now, we pass the client through the
    // CLI handler which has access to the token.
    //
    // This function exists as a placeholder — the actual HTTP client
    // is built in the CLI handler (run_stl.rs) and passed directly.
    //
    // We use the inner client from the Progenitor client.
    Ok(client.client().clone())
}

/// Extract the base URL from the API client.
fn extract_base_url(client: &api::Client) -> String {
    client.baseurl().to_owned()
}

/// Display a JSON value as a string (for IDs).
fn json_value_display(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// Get past tense of an action for output.
fn action_past_tense(action: &Action) -> String {
    match action {
        Action::Create => "Created".to_string(),
        Action::Update => "Updated".to_string(),
        Action::Delete => "Deleted".to_string(),
        Action::Comment => "Added comment to".to_string(),
        Action::Link => "Linked".to_string(),
        Action::Unlink => "Unlinked".to_string(),
        Action::Check => "Checked".to_string(),
        Action::Uncheck => "Unchecked".to_string(),
    }
}

/// Print a success line for a completed operation.
#[allow(clippy::too_many_arguments)]
fn print_success_line(
    out: &OutputConfig,
    counter: usize,
    total: usize,
    action_past: &str,
    action: &Action,
    entity: &Entity,
    response: &serde_json::Value,
    id: Option<&serde_json::Value>,
) -> Result<(), Box<dyn Error>> {
    let id_display = response
        .get("id")
        .map(json_value_display)
        .or_else(|| id.map(json_value_display))
        .unwrap_or_default();
    let name = response.get("name").and_then(|v| v.as_str()).unwrap_or("");

    if action == &Action::Comment || name.is_empty() {
        out_println!(
            out,
            "[{}/{}] {} {} {}",
            counter,
            total,
            action_past,
            entity,
            id_display
        );
    } else {
        out_println!(
            out,
            "[{}/{}] {} {} {} - {}",
            counter,
            total,
            action_past,
            entity,
            id_display,
            name
        );
    }
    Ok(())
}

/// Check if a JSON body is effectively empty (no fields or empty object).
fn json_body_is_empty(body: &serde_json::Value) -> bool {
    match body {
        serde_json::Value::Object(m) => m.is_empty(),
        serde_json::Value::Null => true,
        _ => false,
    }
}

/// Generate a placeholder result for dry-run mode.
fn dry_run_placeholder(entity: &Entity, counter: usize) -> serde_json::Value {
    serde_json::json!({
        "id": counter * 1000,
        "entity_type": entity.to_string(),
    })
}
