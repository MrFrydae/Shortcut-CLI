use std::collections::{HashMap, HashSet};

use super::types::{Action, Entity, Template};

/// A validation error with context.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub operation_index: Option<usize>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(idx) = self.operation_index {
            write!(f, "operation {}: {}", idx + 1, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

/// Validate a parsed template without making any API calls.
///
/// Returns a list of validation errors (empty means valid).
pub fn validate(template: &Template) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // 1. Version must be 1
    if template.version != 1 {
        errors.push(ValidationError {
            message: format!(
                "unsupported version {}; only version 1 is supported",
                template.version
            ),
            operation_index: None,
        });
    }

    // Collect declared variable names
    let declared_vars: HashSet<&str> = template
        .vars
        .as_ref()
        .map(|v| v.keys().map(|k| k.as_str()).collect())
        .unwrap_or_default();

    // Validate variable names
    if let Some(vars) = &template.vars {
        for name in vars.keys() {
            if !is_valid_var_name(name) {
                errors.push(ValidationError {
                    message: format!(
                        "invalid variable name '{name}': must match [a-zA-Z][a-zA-Z0-9_]*"
                    ),
                    operation_index: None,
                });
            }
        }
    }

    // Track aliases defined so far (for forward-reference checking)
    let mut defined_aliases: HashMap<&str, usize> = HashMap::new();

    for (idx, op) in template.operations.iter().enumerate() {
        // 4. Action-entity compatibility
        validate_action_entity(op.action.clone(), op.entity.clone(), idx, &mut errors);

        // 5. Required fields for create (skip when repeat is present —
        //    required fields may come from the repeat entries)
        if op.action == Action::Create && op.repeat.is_none() {
            validate_required_fields_on_create(&op.entity, op.fields.as_ref(), idx, &mut errors);
        }

        // 6. update/delete/comment/unlink/check/uncheck require id
        if matches!(
            op.action,
            Action::Update
                | Action::Delete
                | Action::Comment
                | Action::Unlink
                | Action::Check
                | Action::Uncheck
        ) && op.id.is_none()
        {
            errors.push(ValidationError {
                message: format!("{} action requires an 'id' field", op.action),
                operation_index: Some(idx),
            });
        }

        // 9. Check duplicate aliases
        if let Some(alias) = &op.alias {
            // 10. Alias name format
            if !is_valid_alias_name(alias) {
                errors.push(ValidationError {
                    message: format!(
                        "invalid alias name '{alias}': must match [a-zA-Z][a-zA-Z0-9_-]*"
                    ),
                    operation_index: Some(idx),
                });
            }
            if let Some(&prev_idx) = defined_aliases.get(alias.as_str()) {
                errors.push(ValidationError {
                    message: format!(
                        "duplicate alias '{alias}' (first defined in operation {})",
                        prev_idx + 1
                    ),
                    operation_index: Some(idx),
                });
            } else {
                defined_aliases.insert(alias, idx);
            }
        }

        // 7. Check $ref() targets in id
        if let Some(id_val) = &op.id
            && let Some(ref_name) = extract_ref_from_yaml(id_val)
        {
            let base = ref_base(&ref_name);
            if !defined_aliases.contains_key(base) {
                errors.push(ValidationError {
                    message: format!("$ref({ref_name}) references undefined alias '{base}'"),
                    operation_index: Some(idx),
                });
            }
        }

        // 7+8. Check $ref() and $var() in fields
        if let Some(fields) = &op.fields {
            check_refs_and_vars_in_mapping(
                fields,
                &defined_aliases,
                &declared_vars,
                idx,
                &mut errors,
            );
        }

        // Check $ref() and $var() in repeat entries
        if let Some(repeat) = &op.repeat {
            for entry in repeat {
                check_refs_and_vars_in_mapping(
                    entry,
                    &defined_aliases,
                    &declared_vars,
                    idx,
                    &mut errors,
                );
            }
        }

        // 12. Known fields per entity (warn level — we add as errors for strictness)
        // For comment/link/check/uncheck actions, use the target entity's field list
        if let Some(fields) = &op.fields {
            let field_entity = match op.action {
                Action::Comment => &Entity::Comment,
                Action::Link | Action::Unlink => &Entity::StoryLink,
                Action::Check | Action::Uncheck => &Entity::Task,
                _ => &op.entity,
            };
            check_known_fields(field_entity, fields, idx, &mut errors);
        }
    }

    errors
}

/// Check if a string is a valid alias name: [a-zA-Z][a-zA-Z0-9_-]*
fn is_valid_alias_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Check if a string is a valid variable name: [a-zA-Z][a-zA-Z0-9_]*
fn is_valid_var_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Extract a $ref(name) from a YAML value if it's a pure reference.
fn extract_ref_from_yaml(val: &serde_yaml::Value) -> Option<String> {
    if let serde_yaml::Value::String(s) = val {
        extract_ref(s)
    } else {
        None
    }
}

/// Extract the alias name from a `$ref(name)` string.
fn extract_ref(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.starts_with("$ref(") && trimmed.ends_with(')') {
        Some(trimmed[5..trimmed.len() - 1].to_string())
    } else {
        None
    }
}

/// Get the base alias from a ref like "alias.field" → "alias".
fn ref_base(ref_name: &str) -> &str {
    ref_name.split('.').next().unwrap_or(ref_name)
}

/// Extract all $var(name) references from a string.
fn extract_var_refs(s: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut rest = s;
    while let Some(start) = rest.find("$var(") {
        let after = &rest[start + 5..];
        if let Some(end) = after.find(')') {
            refs.push(after[..end].to_string());
            rest = &after[end + 1..];
        } else {
            break;
        }
    }
    refs
}

/// Extract all $ref(name) references from a string (including inline ones).
fn extract_ref_refs(s: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut rest = s;
    while let Some(start) = rest.find("$ref(") {
        let after = &rest[start + 5..];
        if let Some(end) = after.find(')') {
            refs.push(after[..end].to_string());
            rest = &after[end + 1..];
        } else {
            break;
        }
    }
    refs
}

/// Recursively check $ref() and $var() in a YAML mapping.
fn check_refs_and_vars_in_mapping(
    mapping: &serde_yaml::Mapping,
    defined_aliases: &HashMap<&str, usize>,
    declared_vars: &HashSet<&str>,
    op_idx: usize,
    errors: &mut Vec<ValidationError>,
) {
    for (_key, value) in mapping {
        check_refs_and_vars_in_value(value, defined_aliases, declared_vars, op_idx, errors);
    }
}

/// Recursively check $ref() and $var() in a YAML value.
fn check_refs_and_vars_in_value(
    value: &serde_yaml::Value,
    defined_aliases: &HashMap<&str, usize>,
    declared_vars: &HashSet<&str>,
    op_idx: usize,
    errors: &mut Vec<ValidationError>,
) {
    match value {
        serde_yaml::Value::String(s) => {
            // Check $ref() references
            for ref_name in extract_ref_refs(s) {
                let base = ref_base(&ref_name);
                if !defined_aliases.contains_key(base) {
                    errors.push(ValidationError {
                        message: format!("$ref({ref_name}) references undefined alias '{base}'"),
                        operation_index: Some(op_idx),
                    });
                }
            }
            // Check $var() references
            for var_name in extract_var_refs(s) {
                if !declared_vars.contains(var_name.as_str()) {
                    errors.push(ValidationError {
                        message: format!(
                            "$var({var_name}) references undeclared variable '{var_name}'"
                        ),
                        operation_index: Some(op_idx),
                    });
                }
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for item in seq {
                check_refs_and_vars_in_value(item, defined_aliases, declared_vars, op_idx, errors);
            }
        }
        serde_yaml::Value::Mapping(map) => {
            check_refs_and_vars_in_mapping(map, defined_aliases, declared_vars, op_idx, errors);
        }
        _ => {}
    }
}

/// Validate action-entity compatibility.
fn validate_action_entity(
    action: Action,
    entity: Entity,
    idx: usize,
    errors: &mut Vec<ValidationError>,
) {
    let valid = match action {
        Action::Create => !matches!(entity, Entity::Comment | Entity::StoryLink),
        Action::Update => !matches!(entity, Entity::Comment | Entity::StoryLink | Entity::Task),
        Action::Delete => !matches!(entity, Entity::Comment | Entity::Group),
        Action::Comment => matches!(entity, Entity::Story | Entity::Epic),
        Action::Link => matches!(entity, Entity::StoryLink),
        Action::Unlink => matches!(entity, Entity::StoryLink),
        Action::Check | Action::Uncheck => matches!(entity, Entity::Task),
    };
    if !valid {
        errors.push(ValidationError {
            message: format!("'{action}' action is not valid for '{entity}' entity"),
            operation_index: Some(idx),
        });
    }
}

/// Check that create actions have required fields.
fn validate_required_fields_on_create(
    entity: &Entity,
    fields: Option<&serde_yaml::Mapping>,
    idx: usize,
    errors: &mut Vec<ValidationError>,
) {
    let required: &[&str] = match entity {
        Entity::Story => &["name"],
        Entity::Epic => &["name"],
        Entity::Iteration => &["name", "start_date", "end_date"],
        Entity::Label => &["name"],
        Entity::Objective => &["name"],
        Entity::Milestone => &["name"],
        Entity::Category => &["name"],
        Entity::Group => &["name"],
        Entity::Document => &["name"],
        Entity::Project => &["name"],
        Entity::Task => &["description"],
        _ => &[],
    };

    for &field in required {
        let has_field = fields
            .map(|f| f.get(serde_yaml::Value::String(field.into())).is_some())
            .unwrap_or(false);
        if !has_field {
            errors.push(ValidationError {
                message: format!("create {entity} requires field '{field}'"),
                operation_index: Some(idx),
            });
        }
    }
}

/// Known fields per entity type.
fn known_fields(entity: &Entity) -> &[&str] {
    match entity {
        Entity::Story => &[
            "name",
            "description",
            "type",
            "owner",
            "owners",
            "state",
            "epic_id",
            "iteration_id",
            "project_id",
            "group_id",
            "estimate",
            "labels",
            "followers",
            "requested_by",
            "deadline",
            "custom_fields",
            "tasks",
            "comments",
            "story_links",
        ],
        Entity::Epic => &[
            "name",
            "description",
            "state",
            "deadline",
            "owners",
            "followers",
            "requested_by",
            "labels",
            "objective_ids",
            "milestone_id",
            "group_ids",
            "planned_start_date",
        ],
        Entity::Iteration => &[
            "name",
            "start_date",
            "end_date",
            "description",
            "followers",
            "labels",
            "group_ids",
        ],
        Entity::Label => &["name", "color", "description"],
        Entity::Objective => &["name", "description", "categories"],
        Entity::Milestone => &["name", "description", "categories", "state"],
        Entity::Category => &["name", "color", "type", "description"],
        Entity::Group => &[
            "name",
            "description",
            "member_ids",
            "mention_name",
            "workflow_ids",
        ],
        Entity::Document => &["name", "content", "content_file"],
        Entity::Project => &["name", "description", "team_id", "abbreviation", "color"],
        Entity::Task => &["story_id", "description", "complete", "owners"],
        Entity::Comment => &["story_id", "epic_id", "text", "text_file"],
        Entity::StoryLink => &["subject_id", "object_id", "verb"],
    }
}

/// Check that field names are known for the entity type.
fn check_known_fields(
    entity: &Entity,
    fields: &serde_yaml::Mapping,
    idx: usize,
    errors: &mut Vec<ValidationError>,
) {
    let known = known_fields(entity);
    for key in fields.keys() {
        if let serde_yaml::Value::String(name) = key
            && !known.contains(&name.as_str())
        {
            errors.push(ValidationError {
                message: format!("unknown field '{name}' for {entity} entity"),
                operation_index: Some(idx),
            });
        }
    }
}
