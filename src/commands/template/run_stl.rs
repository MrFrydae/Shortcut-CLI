use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;
use crate::stl::{executor, parser, validator};

#[derive(Args)]
pub struct RunArgs {
    /// Path to the .sc.yml template file (use - for stdin)
    pub file: String,

    /// Skip the interactive confirmation prompt
    #[arg(long)]
    pub confirm: bool,

    /// Pass/override a variable (repeatable, format: key=value)
    #[arg(long = "var", value_parser = parse_var_arg)]
    pub vars: Vec<(String, String)>,
}

fn parse_var_arg(arg: &str) -> Result<(String, String), String> {
    let (key, value) = arg
        .split_once('=')
        .ok_or_else(|| format!("Invalid --var format '{arg}': expected key=value"))?;
    Ok((key.to_string(), value.to_string()))
}

pub async fn run(
    args: &RunArgs,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    // Parse
    let mut template = parser::parse_from_path(&args.file)?;

    // Apply CLI --var overrides
    if !args.vars.is_empty() {
        let vars = template.vars.get_or_insert_with(Default::default);
        for (key, value) in &args.vars {
            if !vars.contains_key(key) {
                return Err(
                    format!("--var '{key}' is not declared in the template's vars block").into(),
                );
            }
            vars.insert(key.clone(), serde_yaml::Value::String(value.clone()));
        }
    }

    // Validate
    let errors = validator::validate(&template);
    if !errors.is_empty() {
        out_println!(out, "Validation errors:");
        for err in &errors {
            out_println!(out, "  - {err}");
        }
        return Err(format!("{} validation error(s) found", errors.len()).into());
    }

    // Execute
    let result = executor::execute(&mut template, client, cache_dir, out, args.confirm).await?;

    // Print summary
    if out.is_json() {
        out_println!(out, "{}", serde_json::to_string_pretty(&result)?);
    } else if !out.is_dry_run() {
        let summary = &result.summary;
        if summary.failed == 0 {
            out_println!(
                out,
                "\nExecuted {}/{} operations successfully.",
                summary.succeeded,
                summary.total
            );
        } else {
            out_println!(
                out,
                "\nExecuted {}/{} operations ({} failed).",
                summary.succeeded,
                summary.total,
                summary.failed
            );
            // List completed operations
            let completed: Vec<&_> = result
                .operations
                .iter()
                .filter(|r| r.status == "success")
                .collect();
            if !completed.is_empty() {
                out_println!(out, "Completed:");
                for op in completed {
                    let id_str = op
                        .result
                        .as_ref()
                        .and_then(|r| r.get("id"))
                        .map(|v| format!(" {}", v))
                        .unwrap_or_default();
                    out_println!(
                        out,
                        "  operation {} ({} {}{})",
                        op.index + 1,
                        op.action,
                        op.entity,
                        id_str
                    );
                }
            }
        }
    }

    if result.summary.failed > 0 {
        Err(format!("{} operation(s) failed", result.summary.failed).into())
    } else {
        Ok(())
    }
}
