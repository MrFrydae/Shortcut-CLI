use std::error::Error;
use std::io::Read;

use super::types::Template;

/// Parse a YAML string into a `Template`.
pub fn parse(yaml: &str) -> Result<Template, Box<dyn Error>> {
    let template: Template =
        serde_yaml::from_str(yaml).map_err(|e| format!("Failed to parse template YAML: {e}"))?;
    Ok(template)
}

/// Read and parse a template from a file path, or from stdin if the path is "-".
///
/// Accepts both YAML (`.shortcut.yml`) and JSON (`.shortcut.json`) files.
/// JSON files are parsed through `serde_yaml` (which handles JSON natively),
/// but on error we re-parse with `serde_json` to produce clearer error messages.
pub fn parse_from_path(path: &str) -> Result<Template, Box<dyn Error>> {
    let content = if path == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("Failed to read from stdin: {e}"))?;
        buf
    } else {
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file '{path}': {e}"))?
    };

    let is_json = path.ends_with(".json");

    match serde_yaml::from_str::<Template>(&content) {
        Ok(template) => Ok(template),
        Err(yaml_err) if is_json => {
            // Re-parse with serde_json for a better error message
            match serde_json::from_str::<serde_json::Value>(&content) {
                Err(json_err) => Err(format!("Failed to parse template JSON: {json_err}").into()),
                Ok(_) => {
                    // Valid JSON but doesn't match the Template schema
                    Err(format!("Failed to parse template JSON: {yaml_err}").into())
                }
            }
        }
        Err(yaml_err) => Err(format!("Failed to parse template YAML: {yaml_err}").into()),
    }
}
