use std::collections::HashMap;

/// Substitute all `$var(name)` references in a YAML value tree.
///
/// Variables are resolved from the `vars` map. Returns errors for any
/// undeclared variable references.
pub fn substitute_vars(
    value: &mut serde_yaml::Value,
    vars: &HashMap<String, serde_yaml::Value>,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    substitute_vars_inner(value, vars, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn substitute_vars_inner(
    value: &mut serde_yaml::Value,
    vars: &HashMap<String, serde_yaml::Value>,
    errors: &mut Vec<String>,
) {
    match value {
        serde_yaml::Value::String(s) => {
            if let Some(resolved) = resolve_var_string(s, vars, errors) {
                *value = resolved;
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for item in seq.iter_mut() {
                substitute_vars_inner(item, vars, errors);
            }
        }
        serde_yaml::Value::Mapping(map) => {
            // Must collect keys first since we can't mutably iterate and modify
            let keys: Vec<serde_yaml::Value> = map.keys().cloned().collect();
            for key in keys {
                if let Some(val) = map.get_mut(&key) {
                    substitute_vars_inner(val, vars, errors);
                }
            }
        }
        _ => {}
    }
}

/// Resolve `$var()` in a string. Returns a new YAML value if substitution occurred.
///
/// If the entire string is `$var(name)` and the variable is a non-string type
/// (integer, boolean, etc.), the raw value is returned (not stringified).
/// For inline interpolation like `"text $var(name) text"`, the variable value
/// is stringified and embedded.
fn resolve_var_string(
    s: &str,
    vars: &HashMap<String, serde_yaml::Value>,
    errors: &mut Vec<String>,
) -> Option<serde_yaml::Value> {
    // Fast path: no $var() at all
    if !s.contains("$var(") {
        return None;
    }

    // Check if the entire string is a single $var(name)
    let trimmed = s.trim();
    if trimmed.starts_with("$var(")
        && trimmed.ends_with(')')
        && trimmed.matches("$var(").count() == 1
    {
        let var_name = &trimmed[5..trimmed.len() - 1];
        if let Some(val) = vars.get(var_name) {
            return Some(val.clone());
        } else {
            errors.push(format!("undeclared variable '{var_name}'"));
            return None;
        }
    }

    // Inline interpolation: replace all $var(name) within the string
    let mut result = s.to_string();
    let mut search_start = 0;
    loop {
        let Some(start) = result[search_start..].find("$var(") else {
            break;
        };
        let abs_start = search_start + start;
        let after = &result[abs_start + 5..];
        let Some(end) = after.find(')') else {
            break;
        };
        let var_name = &after[..end].to_string();
        let abs_end = abs_start + 5 + end + 1; // includes the ')'

        if let Some(val) = vars.get(var_name.as_str()) {
            let replacement = yaml_value_to_string(val);
            result.replace_range(abs_start..abs_end, &replacement);
            search_start = abs_start + replacement.len();
        } else {
            errors.push(format!("undeclared variable '{var_name}'"));
            search_start = abs_end;
        }
    }

    Some(serde_yaml::Value::String(result))
}

/// Convert a YAML value to its string representation for interpolation.
fn yaml_value_to_string(val: &serde_yaml::Value) -> String {
    match val {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Null => "null".to_string(),
        other => format!("{other:?}"),
    }
}

/// Resolve all `$ref(alias)` references in a JSON value tree, replacing them
/// with the resolved values from completed operation results.
///
/// `results` maps alias names to the full JSON response from the API.
/// - `$ref(alias)` → resolved to the entity's primary ID (`id` field)
/// - `$ref(alias.field)` → resolved to a specific field value
/// - `$ref(alias.N)` → for repeat aliases, resolved to the Nth result's ID
pub fn resolve_refs(
    value: &mut serde_json::Value,
    results: &HashMap<String, serde_json::Value>,
) -> Result<(), String> {
    match value {
        serde_json::Value::String(s) => {
            if let Some(resolved) = resolve_ref_string(s, results)? {
                *value = resolved;
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                resolve_refs(item, results)?;
            }
        }
        serde_json::Value::Object(map) => {
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if let Some(val) = map.get_mut(&key) {
                    resolve_refs(val, results)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Resolve `$ref()` in a string. Returns a new JSON value if substitution occurred.
fn resolve_ref_string(
    s: &str,
    results: &HashMap<String, serde_json::Value>,
) -> Result<Option<serde_json::Value>, String> {
    if !s.contains("$ref(") {
        return Ok(None);
    }

    // Check if the entire string is a single $ref(...)
    let trimmed = s.trim();
    if trimmed.starts_with("$ref(")
        && trimmed.ends_with(')')
        && trimmed.matches("$ref(").count() == 1
    {
        let ref_expr = &trimmed[5..trimmed.len() - 1];
        let resolved = resolve_single_ref(ref_expr, results)?;
        return Ok(Some(resolved));
    }

    // Inline interpolation
    let mut result = s.to_string();
    let mut search_start = 0;
    loop {
        let Some(start) = result[search_start..].find("$ref(") else {
            break;
        };
        let abs_start = search_start + start;
        let after = &result[abs_start + 5..];
        let Some(end) = after.find(')') else {
            break;
        };
        let ref_expr = after[..end].to_string();
        let abs_end = abs_start + 5 + end + 1;

        let resolved = resolve_single_ref(&ref_expr, results)?;
        let replacement = json_value_to_string(&resolved);
        result.replace_range(abs_start..abs_end, &replacement);
        search_start = abs_start + replacement.len();
    }

    Ok(Some(serde_json::Value::String(result)))
}

/// Resolve a single $ref expression (without the $ref() wrapper).
fn resolve_single_ref(
    expr: &str,
    results: &HashMap<String, serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let parts: Vec<&str> = expr.splitn(2, '.').collect();
    let alias = parts[0];

    let result = results
        .get(alias)
        .ok_or_else(|| format!("$ref({expr}): alias '{alias}' not found in results"))?;

    if parts.len() == 1 {
        // $ref(alias) → return the primary ID
        return extract_primary_id(result, alias);
    }

    let field = parts[1];

    // Check if it's a numeric index (for repeat aliases with array results)
    if let Ok(index) = field.parse::<usize>() {
        if let serde_json::Value::Array(arr) = result {
            return arr
                .get(index)
                .cloned()
                .ok_or_else(|| {
                    format!(
                        "$ref({expr}): index {index} out of bounds (array has {} elements)",
                        arr.len()
                    )
                })
                .and_then(|v| extract_primary_id(&v, alias));
        } else {
            return Err(format!(
                "$ref({expr}): alias '{alias}' is not an array result"
            ));
        }
    }

    // $ref(alias.field) → return specific field
    if let serde_json::Value::Array(_) = result {
        return Err(format!(
            "$ref({expr}): alias '{alias}' is an array result; use $ref({alias}.N.{field}) for indexed access"
        ));
    }
    result
        .get(field)
        .cloned()
        .ok_or_else(|| format!("$ref({expr}): field '{field}' not found in alias '{alias}' result"))
}

/// Extract the primary ID from an API response.
fn extract_primary_id(value: &serde_json::Value, alias: &str) -> Result<serde_json::Value, String> {
    if let Some(id) = value.get("id") {
        Ok(id.clone())
    } else {
        Err(format!("$ref({alias}): no 'id' field in alias result"))
    }
}

/// Convert a JSON value to string for inline interpolation.
fn json_value_to_string(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

/// Convert a `serde_yaml::Value` to `serde_json::Value`.
pub fn yaml_to_json(yaml: &serde_yaml::Value) -> serde_json::Value {
    match yaml {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(i.into())
            } else if let Some(u) = n.as_u64() {
                serde_json::Value::Number(u.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            serde_json::Value::Array(seq.iter().map(yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                if let serde_yaml::Value::String(key) = k {
                    obj.insert(key.clone(), yaml_to_json(v));
                }
            }
            serde_json::Value::Object(obj)
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(&tagged.value),
    }
}

/// Convert a `serde_yaml::Mapping` to `serde_json::Value::Object`.
pub fn yaml_mapping_to_json(mapping: &serde_yaml::Mapping) -> serde_json::Value {
    yaml_to_json(&serde_yaml::Value::Mapping(mapping.clone()))
}
