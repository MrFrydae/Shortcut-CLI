use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

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

                // Extract story_id before resolve_entity_fields removes it.
                let story_id = json_body.get("story_id").and_then(|v| v.as_i64());

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
                        out_println!(
                            out,
                            "[{}/{}] {} {}",
                            op_counter,
                            total,
                            op.action,
                            op.entity
                        );
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

                // Execute the API call via generated client
                match dispatch_api_call(client, &op.action, &op.entity, None, story_id, json_body)
                    .await
                {
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

            // Extract story_id before resolve_entity_fields removes it.
            let story_id = json_body.get("story_id").and_then(|v| v.as_i64());

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
                    out_println!(
                        out,
                        "[{}/{}] {} {}",
                        op_counter,
                        total,
                        op.action,
                        op.entity
                    );
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

            // Execute the API call via generated client
            match dispatch_api_call(
                client,
                &op.action,
                &op.entity,
                resolved_id.as_ref(),
                story_id,
                json_body,
            )
            .await
            {
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
///
/// The Shortcut API only accepts three verbs: "blocks", "duplicates", "relates to".
/// "blocked-by" variants map to "blocks" — users should use subject/object ordering
/// to express directionality.
fn normalize_link_verb(verb: &str) -> String {
    match verb.to_lowercase().as_str() {
        "blocks" | "blocked-by" | "blocked_by" | "is blocked by" => "blocks".to_string(),
        "duplicates" => "duplicates".to_string(),
        "relates-to" | "relates_to" | "relates to" => "relates to".to_string(),
        other => other.to_string(),
    }
}

/// Dispatch an API call through the generated Progenitor client.
async fn dispatch_api_call(
    client: &api::Client,
    action: &Action,
    entity: &Entity,
    id: Option<&serde_json::Value>,
    story_id: Option<i64>,
    body: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn Error>> {
    match (action, entity) {
        // ── Story ──
        (Action::Create, Entity::Story) => {
            let p: api::types::CreateStoryParams = serde_json::from_value(body)?;
            let r = client
                .create_story()
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Update, Entity::Story) => {
            let id = extract_i64_id(id, "update story requires id")?;
            let p: api::types::UpdateStory = serde_json::from_value(body)?;
            let r = client
                .update_story()
                .story_public_id(id)
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Delete, Entity::Story) => {
            let id = extract_i64_id(id, "delete story requires id")?;
            client
                .delete_story()
                .story_public_id(id)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            Ok(serde_json::json!({}))
        }

        // ── Epic ──
        (Action::Create, Entity::Epic) => {
            let p: api::types::CreateEpic = serde_json::from_value(body)?;
            let r = client
                .create_epic()
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Update, Entity::Epic) => {
            let id = extract_i64_id(id, "update epic requires id")?;
            let p: api::types::UpdateEpic = serde_json::from_value(body)?;
            let r = client
                .update_epic()
                .epic_public_id(id)
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Delete, Entity::Epic) => {
            let id = extract_i64_id(id, "delete epic requires id")?;
            client
                .delete_epic()
                .epic_public_id(id)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            Ok(serde_json::json!({}))
        }

        // ── Iteration ──
        (Action::Create, Entity::Iteration) => {
            let p: api::types::CreateIteration = serde_json::from_value(body)?;
            let r = client
                .create_iteration()
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Update, Entity::Iteration) => {
            let id = extract_i64_id(id, "update iteration requires id")?;
            let p: api::types::UpdateIteration = serde_json::from_value(body)?;
            let r = client
                .update_iteration()
                .iteration_public_id(id)
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Delete, Entity::Iteration) => {
            let id = extract_i64_id(id, "delete iteration requires id")?;
            client
                .delete_iteration()
                .iteration_public_id(id)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            Ok(serde_json::json!({}))
        }

        // ── Label ──
        (Action::Create, Entity::Label) => {
            let p: api::types::CreateLabelParams = serde_json::from_value(body)?;
            let r = client
                .create_label()
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Update, Entity::Label) => {
            let id = extract_i64_id(id, "update label requires id")?;
            let p: api::types::UpdateLabel = serde_json::from_value(body)?;
            let r = client
                .update_label()
                .label_public_id(id)
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Delete, Entity::Label) => {
            let id = extract_i64_id(id, "delete label requires id")?;
            client
                .delete_label()
                .label_public_id(id)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            Ok(serde_json::json!({}))
        }

        // ── Objective ──
        (Action::Create, Entity::Objective) => {
            let p: api::types::CreateObjective = serde_json::from_value(body)?;
            let r = client
                .create_objective()
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Update, Entity::Objective) => {
            let id = extract_i64_id(id, "update objective requires id")?;
            let p: api::types::UpdateObjective = serde_json::from_value(body)?;
            let r = client
                .update_objective()
                .objective_public_id(id)
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Delete, Entity::Objective) => {
            let id = extract_i64_id(id, "delete objective requires id")?;
            client
                .delete_objective()
                .objective_public_id(id)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            Ok(serde_json::json!({}))
        }

        // ── Milestone ──
        (Action::Create, Entity::Milestone) => {
            let p: api::types::CreateMilestone = serde_json::from_value(body)?;
            let r = client
                .create_milestone()
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Update, Entity::Milestone) => {
            let id = extract_i64_id(id, "update milestone requires id")?;
            let p: api::types::UpdateMilestone = serde_json::from_value(body)?;
            let r = client
                .update_milestone()
                .milestone_public_id(id)
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Delete, Entity::Milestone) => {
            let id = extract_i64_id(id, "delete milestone requires id")?;
            client
                .delete_milestone()
                .milestone_public_id(id)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            Ok(serde_json::json!({}))
        }

        // ── Category ──
        (Action::Create, Entity::Category) => {
            let p: api::types::CreateCategory = serde_json::from_value(body)?;
            let r = client
                .create_category()
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Update, Entity::Category) => {
            let id = extract_i64_id(id, "update category requires id")?;
            let p: api::types::UpdateCategory = serde_json::from_value(body)?;
            let r = client
                .update_category()
                .category_public_id(id)
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Delete, Entity::Category) => {
            let id = extract_i64_id(id, "delete category requires id")?;
            client
                .delete_category()
                .category_public_id(id)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            Ok(serde_json::json!({}))
        }

        // ── Group (UUID id) ──
        (Action::Create, Entity::Group) => {
            let p: api::types::CreateGroup = serde_json::from_value(body)?;
            let r = client
                .create_group()
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Update, Entity::Group) => {
            let id = extract_uuid_id(id, "update group requires id")?;
            let p: api::types::UpdateGroup = serde_json::from_value(body)?;
            let r = client
                .update_group()
                .group_public_id(id)
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }

        // ── Document (UUID id) ──
        (Action::Create, Entity::Document) => {
            let p: api::types::CreateDoc = serde_json::from_value(body)?;
            let r = client
                .create_doc()
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Update, Entity::Document) => {
            let id = extract_uuid_id(id, "update document requires id")?;
            let p: api::types::UpdateDoc = serde_json::from_value(body)?;
            let r = client
                .update_doc()
                .doc_public_id(id)
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Delete, Entity::Document) => {
            let id = extract_uuid_id(id, "delete document requires id")?;
            client
                .delete_doc()
                .doc_public_id(id)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            Ok(serde_json::json!({}))
        }

        // ── Project ──
        (Action::Create, Entity::Project) => {
            let p: api::types::CreateProject = serde_json::from_value(body)?;
            let r = client
                .create_project()
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Update, Entity::Project) => {
            let id = extract_i64_id(id, "update project requires id")?;
            let p: api::types::UpdateProject = serde_json::from_value(body)?;
            let r = client
                .update_project()
                .project_public_id(id)
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Delete, Entity::Project) => {
            let id = extract_i64_id(id, "delete project requires id")?;
            client
                .delete_project()
                .project_public_id(id)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            Ok(serde_json::json!({}))
        }

        // ── Task (needs story_public_id path param) ──
        (Action::Create, Entity::Task) => {
            let sid = story_id.ok_or("create task requires story_id")?;
            let p: api::types::CreateTask = serde_json::from_value(body)?;
            let r = client
                .create_task()
                .story_public_id(sid)
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Check | Action::Uncheck, Entity::Task) => {
            let sid = story_id.ok_or("check/uncheck task requires story_id")?;
            let tid = extract_i64_id(id, "check/uncheck task requires id")?;
            let p: api::types::UpdateTask = serde_json::from_value(body)?;
            let r = client
                .update_task()
                .story_public_id(sid)
                .task_public_id(tid)
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }

        // ── Comment ──
        (Action::Comment, Entity::Story) => {
            let id = extract_i64_id(id, "comment on story requires id")?;
            let p: api::types::CreateStoryComment = serde_json::from_value(body)?;
            let r = client
                .create_story_comment()
                .story_public_id(id)
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Comment, Entity::Epic) => {
            let id = extract_i64_id(id, "comment on epic requires id")?;
            let p: api::types::CreateEpicComment = serde_json::from_value(body)?;
            let r = client
                .create_epic_comment()
                .epic_public_id(id)
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }

        // ── Story Link ──
        (Action::Link, Entity::StoryLink) => {
            let p: api::types::CreateStoryLink = serde_json::from_value(body)?;
            let r = client
                .create_story_link()
                .body(p)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            serde_json::to_value(&*r).map_err(Into::into)
        }
        (Action::Unlink, Entity::StoryLink) => {
            let id = extract_i64_id(id, "unlink story-link requires id")?;
            client
                .delete_story_link()
                .story_link_public_id(id)
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            Ok(serde_json::json!({}))
        }

        _ => Err(format!(
            "unsupported action/entity combination: {} {}",
            action, entity
        )
        .into()),
    }
}

/// Extract an i64 ID from a JSON value.
fn extract_i64_id(val: Option<&serde_json::Value>, context: &str) -> Result<i64, Box<dyn Error>> {
    val.and_then(|v| v.as_i64()).ok_or_else(|| context.into())
}

/// Extract a UUID ID from a JSON value.
fn extract_uuid_id(
    val: Option<&serde_json::Value>,
    context: &str,
) -> Result<uuid::Uuid, Box<dyn Error>> {
    let s = val
        .and_then(|v| v.as_str())
        .ok_or_else(|| -> Box<dyn Error> { context.into() })?;
    uuid::Uuid::parse_str(s).map_err(|e| -> Box<dyn Error> { format!("{context}: {e}").into() })
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
