use std::collections::HashSet;

use super::state::{ResourceState, SyncState};
use super::types::{Action, Operation};

/// A planned sync action produced by reconciliation.
#[derive(Debug, Clone)]
pub enum SyncAction {
    /// Single create op, alias not in state -> POST
    Create { op_index: usize, alias: String },
    /// Single create op, alias in state -> PATCH with stored ID
    Update {
        op_index: usize,
        alias: String,
        existing_id: serde_json::Value,
    },
    /// Repeat entry with key not in state -> POST
    CreateEntry {
        op_index: usize,
        alias: String,
        key: String,
    },
    /// Repeat entry with key in state -> PATCH with stored ID
    UpdateEntry {
        op_index: usize,
        alias: String,
        key: String,
        existing_id: serde_json::Value,
    },
    /// Repeat entry key in state but removed from template
    OrphanEntry {
        alias: String,
        key: String,
        entity: String,
        id: serde_json::Value,
    },
    /// Imperative op already applied -> skip
    Skip { op_index: usize, reason: String },
    /// Imperative op not yet applied -> execute
    RunSideEffect { op_index: usize },
    /// Explicit update/delete op -> run as-is
    Passthrough { op_index: usize },
    /// Whole alias in state but not in template
    Orphan {
        alias: String,
        entity: String,
        ids: Vec<serde_json::Value>,
    },
}

impl SyncAction {
    pub fn summary_verb(&self) -> &'static str {
        match self {
            SyncAction::Create { .. } | SyncAction::CreateEntry { .. } => "create",
            SyncAction::Update { .. } | SyncAction::UpdateEntry { .. } => "update",
            SyncAction::OrphanEntry { .. } | SyncAction::Orphan { .. } => "orphan",
            SyncAction::Skip { .. } => "skip",
            SyncAction::RunSideEffect { .. } => "execute",
            SyncAction::Passthrough { .. } => "passthrough",
        }
    }
}

/// Classify a side-effect action type.
fn is_side_effect(action: &Action) -> bool {
    matches!(
        action,
        Action::Comment | Action::Link | Action::Unlink | Action::Check | Action::Uncheck
    )
}

/// Build a side-effect key for the applied set.
fn side_effect_key(op_index: usize, action: &Action) -> String {
    format!("op-{}-{}", op_index, action)
}

/// Compare template operations against state and produce a sync plan.
pub fn reconcile(
    operations: &[Operation],
    state: &Option<SyncState>,
) -> Result<Vec<SyncAction>, String> {
    let mut actions = Vec::new();

    // Track which aliases in state are covered by the template
    let mut seen_aliases: HashSet<String> = HashSet::new();

    for (idx, op) in operations.iter().enumerate() {
        if op.action == Action::Create {
            let alias = op
                .alias
                .as_ref()
                .ok_or_else(|| {
                    format!(
                        "operation {}: sync requires an 'alias' on every create operation",
                        idx + 1
                    )
                })?
                .clone();

            seen_aliases.insert(alias.clone());

            if let Some(repeat) = &op.repeat {
                // Repeat operation: reconcile by key
                reconcile_repeat(
                    idx,
                    &alias,
                    repeat,
                    &op.entity.to_string(),
                    state,
                    &mut actions,
                )?;
            } else {
                // Single operation
                reconcile_single(idx, &alias, &op.entity.to_string(), state, &mut actions)?;
            }
        } else if is_side_effect(&op.action) {
            let key = side_effect_key(idx, &op.action);
            let already_applied = state
                .as_ref()
                .map(|s| s.applied.contains(&key))
                .unwrap_or(false);

            if already_applied {
                actions.push(SyncAction::Skip {
                    op_index: idx,
                    reason: format!("{} already applied", op.action),
                });
            } else {
                actions.push(SyncAction::RunSideEffect { op_index: idx });
            }
        } else {
            // Update/Delete with explicit id — pass through, not state-managed
            actions.push(SyncAction::Passthrough { op_index: idx });
        }
    }

    // Find orphaned aliases (in state but not in template)
    if let Some(st) = state {
        for (alias, resource) in &st.resources {
            if !seen_aliases.contains(alias) {
                let entity = resource.entity().to_string();
                let ids = match resource {
                    ResourceState::Single { id, .. } => vec![id.clone()],
                    ResourceState::Repeat { entries, .. } => {
                        entries.values().map(|e| e.id.clone()).collect()
                    }
                };
                actions.push(SyncAction::Orphan {
                    alias: alias.clone(),
                    entity,
                    ids,
                });
            }
        }
    }

    Ok(actions)
}

fn reconcile_single(
    op_index: usize,
    alias: &str,
    entity: &str,
    state: &Option<SyncState>,
    actions: &mut Vec<SyncAction>,
) -> Result<(), String> {
    if let Some(st) = state {
        if let Some(resource) = st.resources.get(alias) {
            // Validate entity type hasn't changed
            if resource.entity() != entity {
                return Err(format!(
                    "operation {}: entity type changed from '{}' to '{entity}' for alias '{alias}'",
                    op_index + 1,
                    resource.entity()
                ));
            }
            match resource {
                ResourceState::Single { id, .. } => {
                    actions.push(SyncAction::Update {
                        op_index,
                        alias: alias.to_string(),
                        existing_id: id.clone(),
                    });
                }
                ResourceState::Repeat { .. } => {
                    return Err(format!(
                        "operation {}: alias '{alias}' was a repeat operation in state but is now single",
                        op_index + 1
                    ));
                }
            }
        } else {
            actions.push(SyncAction::Create {
                op_index,
                alias: alias.to_string(),
            });
        }
    } else {
        actions.push(SyncAction::Create {
            op_index,
            alias: alias.to_string(),
        });
    }
    Ok(())
}

fn reconcile_repeat(
    op_index: usize,
    alias: &str,
    repeat: &[serde_yaml::Mapping],
    entity: &str,
    state: &Option<SyncState>,
    actions: &mut Vec<SyncAction>,
) -> Result<(), String> {
    // Collect keys from template
    let mut template_keys: Vec<String> = Vec::new();
    for (entry_idx, entry) in repeat.iter().enumerate() {
        let key = entry
            .get(serde_yaml::Value::String("key".to_string()))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                format!(
                    "operation {}: repeat entry {} is missing required 'key' field for sync",
                    op_index + 1,
                    entry_idx + 1
                )
            })?;
        template_keys.push(key);
    }

    if let Some(st) = state {
        if let Some(resource) = st.resources.get(alias) {
            if resource.entity() != entity {
                return Err(format!(
                    "operation {}: entity type changed from '{}' to '{entity}' for alias '{alias}'",
                    op_index + 1,
                    resource.entity()
                ));
            }
            match resource {
                ResourceState::Repeat { entries, .. } => {
                    let template_key_set: HashSet<&str> =
                        template_keys.iter().map(|k| k.as_str()).collect();

                    // For each template key, decide create or update
                    for key in &template_keys {
                        if let Some(entry) = entries.get(key) {
                            actions.push(SyncAction::UpdateEntry {
                                op_index,
                                alias: alias.to_string(),
                                key: key.clone(),
                                existing_id: entry.id.clone(),
                            });
                        } else {
                            actions.push(SyncAction::CreateEntry {
                                op_index,
                                alias: alias.to_string(),
                                key: key.clone(),
                            });
                        }
                    }

                    // Find orphan entries (in state but not in template)
                    for (key, entry) in entries {
                        if !template_key_set.contains(key.as_str()) {
                            actions.push(SyncAction::OrphanEntry {
                                alias: alias.to_string(),
                                key: key.clone(),
                                entity: entity.to_string(),
                                id: entry.id.clone(),
                            });
                        }
                    }
                }
                ResourceState::Single { .. } => {
                    return Err(format!(
                        "operation {}: alias '{alias}' was a single operation in state but is now repeat",
                        op_index + 1
                    ));
                }
            }
        } else {
            // New repeat alias — create all entries
            for key in &template_keys {
                actions.push(SyncAction::CreateEntry {
                    op_index,
                    alias: alias.to_string(),
                    key: key.clone(),
                });
            }
        }
    } else {
        // No state at all — create all entries
        for key in &template_keys {
            actions.push(SyncAction::CreateEntry {
                op_index,
                alias: alias.to_string(),
                key: key.clone(),
            });
        }
    }

    Ok(())
}
