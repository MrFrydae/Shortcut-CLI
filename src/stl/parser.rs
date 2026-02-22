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
pub fn parse_from_path(path: &str) -> Result<Template, Box<dyn Error>> {
    let yaml = if path == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("Failed to read from stdin: {e}"))?;
        buf
    } else {
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file '{path}': {e}"))?
    };
    parse(&yaml)
}
