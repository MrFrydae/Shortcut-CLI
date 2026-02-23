use std::process::Command;
use std::{env, fs, path::Path};

fn main() {
    // Point git to our version-controlled hooks directory.
    // This mirrors npm prepare / husky — anyone who clones and builds
    // gets hooks set up automatically.
    let _ = Command::new("git")
        .args(["config", "core.hooksPath", ".githooks"])
        .status();

    // --- Progenitor: generate Shortcut API client ---
    let spec_path = "spec/shortcut.openapi.json";
    let spec = fs::read_to_string(spec_path).expect("failed to read OpenAPI spec");
    let mut spec_value: serde_json::Value =
        serde_json::from_str(&spec).expect("failed to parse OpenAPI spec");

    // Add a unified ApiError schema so Progenitor deserializes error bodies
    // instead of discarding them. Every Shortcut API error includes a `message` field.
    if let Some(schemas) = spec_value
        .get_mut("components")
        .and_then(|c| c.get_mut("schemas"))
        .and_then(|s| s.as_object_mut())
    {
        schemas.insert(
            "ApiError".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                },
                "required": ["message"]
            }),
        );
    }

    // Patch the spec to work around Progenitor limitations.
    if let Some(paths) = spec_value.get_mut("paths").and_then(|p| p.as_object_mut()) {
        // Remove endpoints that use multipart/form-data (unsupported by Progenitor).
        paths.remove("/api/v3/files");

        // Replace non-2xx response content with a $ref to ApiError.
        // Progenitor asserts at most one error type per operation; using a single
        // shared schema satisfies that constraint while preserving error messages.
        let api_error_content = serde_json::json!({
            "application/json": {
                "schema": {
                    "$ref": "#/components/schemas/ApiError"
                }
            }
        });
        for path_item in paths.values_mut() {
            if let Some(methods) = path_item.as_object_mut() {
                for op in methods.values_mut() {
                    if let Some(responses) = op.get_mut("responses").and_then(|r| r.as_object_mut())
                    {
                        for (code, resp) in responses.iter_mut() {
                            if !code.starts_with('2')
                                && let Some(obj) = resp.as_object_mut()
                            {
                                obj.insert("content".to_string(), api_error_content.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // Mark `associated_groups` as nullable across all schemas.
    // The Shortcut API returns `null` for this field despite the spec
    // declaring it as a required non-nullable array.
    if let Some(schemas) = spec_value
        .get_mut("components")
        .and_then(|c| c.get_mut("schemas"))
        .and_then(|s| s.as_object_mut())
    {
        for (_name, schema) in schemas.iter_mut() {
            if let Some(prop) = schema
                .get_mut("properties")
                .and_then(|p| p.get_mut("associated_groups"))
                .and_then(|ag| ag.as_object_mut())
            {
                prop.insert("nullable".to_string(), serde_json::Value::Bool(true));
            }
        }
    }

    // Make `display_icon` optional in the `Profile` schema.
    // The Shortcut API returns `null` for members without a custom icon,
    // but the spec declares it as a required `$ref` to `Icon`.
    // Removing it from `required` causes Progenitor to generate
    // `Option<Icon>` and accept both `null` and missing values.
    if let Some(required) = spec_value
        .get_mut("components")
        .and_then(|c| c.get_mut("schemas"))
        .and_then(|s| s.get_mut("Profile"))
        .and_then(|p| p.get_mut("required"))
        .and_then(|r| r.as_array_mut())
    {
        required.retain(|v| v.as_str() != Some("display_icon"));
    }

    // Make `display_icon` optional in the `Group` schema (same issue as Profile).
    if let Some(required) = spec_value
        .get_mut("components")
        .and_then(|c| c.get_mut("schemas"))
        .and_then(|s| s.get_mut("Group"))
        .and_then(|p| p.get_mut("required"))
        .and_then(|r| r.as_array_mut())
    {
        required.retain(|v| v.as_str() != Some("display_icon"));
    }

    // Fix PullRequestLabel `id` type.
    // The Shortcut API returns VCS label IDs as strings (GitHub label IDs),
    // but the spec declares them as int64.
    if let Some(id_prop) = spec_value
        .get_mut("components")
        .and_then(|c| c.get_mut("schemas"))
        .and_then(|s| s.get_mut("PullRequestLabel"))
        .and_then(|p| p.get_mut("properties"))
        .and_then(|p| p.get_mut("id"))
        .and_then(|id| id.as_object_mut())
    {
        id_prop.insert(
            "type".to_string(),
            serde_json::Value::String("string".to_string()),
        );
        id_prop.remove("format");
    }

    let spec: openapiv3::OpenAPI =
        serde_json::from_value(spec_value).expect("failed to deserialize OpenAPI spec");

    let mut settings = progenitor::GenerationSettings::default();
    settings.with_interface(progenitor::InterfaceStyle::Builder);

    let mut generator = progenitor::Generator::new(&settings);
    let tokens = generator
        .generate_tokens(&spec)
        .expect("failed to generate API client");
    let ast = syn::parse2(tokens).expect("failed to parse generated tokens");
    let code = prettyplease::unparse(&ast);

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let out_path = Path::new(&out_dir).join("shortcut_api.rs");
    fs::write(&out_path, code).expect("failed to write generated API client");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={spec_path}");
}
