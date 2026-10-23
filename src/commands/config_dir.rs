use std::error::Error;

use crate::output::{OutputConfig, format_template};
use crate::project::ProjectRoot;

/// Print storage paths without reading the token or changing the project.
pub fn run(root: &ProjectRoot, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let paths = serde_json::json!({
        "config_dir": root.config_dir(),
        "token_path": root.token_path(),
        "cache_dir": root.cache_dir(),
    });

    if out.is_machine_readable() {
        out.writeln(format_args!("{}", serde_json::to_string_pretty(&paths)?))?;
    } else if let Some(template) = out.format_template() {
        out.writeln(format_args!("{}", format_template(template, &paths)?))?;
    } else {
        out.writeln(format_args!("{}", root.config_dir().display()))?;
    }
    Ok(())
}
