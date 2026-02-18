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

    // Patch the spec to work around Progenitor limitations.
    if let Some(paths) = spec_value.get_mut("paths").and_then(|p| p.as_object_mut()) {
        // Remove endpoints that use multipart/form-data (unsupported by Progenitor).
        paths.remove("/api/v3/files");

        // Strip `content` from non-2xx responses. Progenitor asserts at most one
        // response type per operation; typed error bodies (400, 403, 409) violate that.
        for path_item in paths.values_mut() {
            if let Some(methods) = path_item.as_object_mut() {
                for op in methods.values_mut() {
                    if let Some(responses) = op.get_mut("responses").and_then(|r| r.as_object_mut())
                    {
                        for (code, resp) in responses.iter_mut() {
                            if !code.starts_with('2')
                                && let Some(obj) = resp.as_object_mut()
                            {
                                obj.remove("content");
                            }
                        }
                    }
                }
            }
        }
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
