use std::collections::HashMap;

use sc::stl::resolver::{resolve_refs, substitute_vars, yaml_to_json};

// --- Variable substitution tests ---

#[test]
fn substitute_simple_var() {
    let mut vars = HashMap::new();
    vars.insert(
        "name".to_string(),
        serde_yaml::Value::String("Sprint 24".into()),
    );

    let mut val = serde_yaml::Value::String("$var(name)".into());
    substitute_vars(&mut val, &vars).unwrap();
    assert_eq!(val, serde_yaml::Value::String("Sprint 24".into()));
}

#[test]
fn substitute_inline_interpolation() {
    let mut vars = HashMap::new();
    vars.insert(
        "name".to_string(),
        serde_yaml::Value::String("Sprint 24".into()),
    );

    let mut val = serde_yaml::Value::String("Focus for $var(name) this week".into());
    substitute_vars(&mut val, &vars).unwrap();
    assert_eq!(
        val,
        serde_yaml::Value::String("Focus for Sprint 24 this week".into())
    );
}

#[test]
fn substitute_numeric_var() {
    let mut vars = HashMap::new();
    vars.insert(
        "estimate".to_string(),
        serde_yaml::Value::Number(serde_yaml::Number::from(5)),
    );

    // When entire value is $var(estimate), preserve the numeric type
    let mut val = serde_yaml::Value::String("$var(estimate)".into());
    substitute_vars(&mut val, &vars).unwrap();
    assert_eq!(val, serde_yaml::Value::Number(serde_yaml::Number::from(5)));
}

#[test]
fn substitute_numeric_inline() {
    let mut vars = HashMap::new();
    vars.insert(
        "count".to_string(),
        serde_yaml::Value::Number(serde_yaml::Number::from(3)),
    );

    // Inline: numeric is stringified
    let mut val = serde_yaml::Value::String("Need $var(count) items".into());
    substitute_vars(&mut val, &vars).unwrap();
    assert_eq!(val, serde_yaml::Value::String("Need 3 items".into()));
}

#[test]
fn substitute_in_sequence() {
    let mut vars = HashMap::new();
    vars.insert(
        "label".to_string(),
        serde_yaml::Value::String("backend".into()),
    );

    let mut val = serde_yaml::Value::Sequence(vec![
        serde_yaml::Value::String("$var(label)".into()),
        serde_yaml::Value::String("frontend".into()),
    ]);
    substitute_vars(&mut val, &vars).unwrap();
    if let serde_yaml::Value::Sequence(seq) = &val {
        assert_eq!(seq[0], serde_yaml::Value::String("backend".into()));
        assert_eq!(seq[1], serde_yaml::Value::String("frontend".into()));
    } else {
        panic!("expected sequence");
    }
}

#[test]
fn substitute_in_mapping() {
    let mut vars = HashMap::new();
    vars.insert(
        "team".to_string(),
        serde_yaml::Value::String("@backend".into()),
    );

    let mut mapping = serde_yaml::Mapping::new();
    mapping.insert(
        serde_yaml::Value::String("group_id".into()),
        serde_yaml::Value::String("$var(team)".into()),
    );
    let mut val = serde_yaml::Value::Mapping(mapping);
    substitute_vars(&mut val, &vars).unwrap();

    if let serde_yaml::Value::Mapping(m) = &val {
        assert_eq!(
            m.get(serde_yaml::Value::String("group_id".into())),
            Some(&serde_yaml::Value::String("@backend".into()))
        );
    }
}

#[test]
fn substitute_undeclared_var_error() {
    let vars = HashMap::new();
    let mut val = serde_yaml::Value::String("$var(missing)".into());
    let result = substitute_vars(&mut val, &vars);
    assert!(result.is_err());
    assert!(result.unwrap_err()[0].contains("undeclared variable 'missing'"));
}

#[test]
fn substitute_multiple_vars_in_one_string() {
    let mut vars = HashMap::new();
    vars.insert(
        "start".to_string(),
        serde_yaml::Value::String("2026-03-02".into()),
    );
    vars.insert(
        "end".to_string(),
        serde_yaml::Value::String("2026-03-15".into()),
    );

    let mut val = serde_yaml::Value::String("From $var(start) to $var(end)".into());
    substitute_vars(&mut val, &vars).unwrap();
    assert_eq!(
        val,
        serde_yaml::Value::String("From 2026-03-02 to 2026-03-15".into())
    );
}

// --- Reference resolution tests ---

#[test]
fn resolve_ref_to_id() {
    let mut results = HashMap::new();
    results.insert(
        "my-epic".to_string(),
        serde_json::json!({"id": 55, "name": "Auth"}),
    );

    let mut val = serde_json::Value::String("$ref(my-epic)".into());
    resolve_refs(&mut val, &results).unwrap();
    assert_eq!(val, serde_json::json!(55));
}

#[test]
fn resolve_ref_field_access() {
    let mut results = HashMap::new();
    results.insert(
        "my-epic".to_string(),
        serde_json::json!({
            "id": 55,
            "name": "Auth",
            "app_url": "https://shortcut.com/epic/55"
        }),
    );

    let mut val = serde_json::Value::String("$ref(my-epic.app_url)".into());
    resolve_refs(&mut val, &results).unwrap();
    assert_eq!(
        val,
        serde_json::Value::String("https://shortcut.com/epic/55".into())
    );
}

#[test]
fn resolve_ref_inline_interpolation() {
    let mut results = HashMap::new();
    results.insert(
        "my-epic".to_string(),
        serde_json::json!({"id": 55, "app_url": "https://shortcut.com/epic/55"}),
    );

    let mut val = serde_json::Value::String("See $ref(my-epic.app_url) for details".into());
    resolve_refs(&mut val, &results).unwrap();
    assert_eq!(
        val,
        serde_json::Value::String("See https://shortcut.com/epic/55 for details".into())
    );
}

#[test]
fn resolve_ref_array_index() {
    let mut results = HashMap::new();
    results.insert(
        "stories".to_string(),
        serde_json::json!([
            {"id": 100, "name": "Story A"},
            {"id": 101, "name": "Story B"}
        ]),
    );

    let mut val = serde_json::Value::String("$ref(stories.0)".into());
    resolve_refs(&mut val, &results).unwrap();
    assert_eq!(val, serde_json::json!(100));

    let mut val = serde_json::Value::String("$ref(stories.1)".into());
    resolve_refs(&mut val, &results).unwrap();
    assert_eq!(val, serde_json::json!(101));
}

#[test]
fn resolve_ref_undefined_alias() {
    let results = HashMap::new();
    let mut val = serde_json::Value::String("$ref(nonexistent)".into());
    let result = resolve_refs(&mut val, &results);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found in results"));
}

#[test]
fn resolve_ref_in_array() {
    let mut results = HashMap::new();
    results.insert("epic".to_string(), serde_json::json!({"id": 55}));

    let mut val = serde_json::json!(["$ref(epic)", "other"]);
    resolve_refs(&mut val, &results).unwrap();
    assert_eq!(val, serde_json::json!([55, "other"]));
}

#[test]
fn resolve_ref_in_object() {
    let mut results = HashMap::new();
    results.insert("sprint".to_string(), serde_json::json!({"id": 10}));

    let mut val = serde_json::json!({"iteration_id": "$ref(sprint)"});
    resolve_refs(&mut val, &results).unwrap();
    assert_eq!(val, serde_json::json!({"iteration_id": 10}));
}

// --- YAML to JSON conversion tests ---

#[test]
fn yaml_to_json_basic_types() {
    assert_eq!(
        yaml_to_json(&serde_yaml::Value::Null),
        serde_json::Value::Null
    );
    assert_eq!(
        yaml_to_json(&serde_yaml::Value::Bool(true)),
        serde_json::Value::Bool(true)
    );
    assert_eq!(
        yaml_to_json(&serde_yaml::Value::String("hello".into())),
        serde_json::Value::String("hello".into())
    );
    assert_eq!(
        yaml_to_json(&serde_yaml::Value::Number(serde_yaml::Number::from(42))),
        serde_json::json!(42)
    );
}

#[test]
fn yaml_to_json_nested() {
    let mut mapping = serde_yaml::Mapping::new();
    mapping.insert(
        serde_yaml::Value::String("name".into()),
        serde_yaml::Value::String("Test".into()),
    );
    mapping.insert(
        serde_yaml::Value::String("labels".into()),
        serde_yaml::Value::Sequence(vec![
            serde_yaml::Value::String("a".into()),
            serde_yaml::Value::String("b".into()),
        ]),
    );

    let json = yaml_to_json(&serde_yaml::Value::Mapping(mapping));
    assert_eq!(
        json,
        serde_json::json!({"name": "Test", "labels": ["a", "b"]})
    );
}
