use std::error::Error;
use std::path::Path;

use clap::Args;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;
use crate::stl::{executor, parser, reconciler, state, validator};

#[derive(Args)]
pub struct SyncArgs {
    /// Path to the .shortcut.yml or .shortcut.json template file
    pub file: String,

    /// Override state file location (default: file.state.json)
    #[arg(long)]
    pub state: Option<String>,

    /// Skip the interactive confirmation prompt
    #[arg(long)]
    pub confirm: bool,

    /// Delete resources that exist in state but were removed from the template
    #[arg(long)]
    pub prune: bool,

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
    args: &SyncArgs,
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

    // Validate (standard + sync-specific)
    let mut errors = validator::validate(&template);
    errors.extend(validator::validate_for_sync(&template));
    if !errors.is_empty() {
        out_println!(out, "Validation errors:");
        for err in &errors {
            out_println!(out, "  - {err}");
        }
        return Err(format!("{} validation error(s) found", errors.len()).into());
    }

    // Load state
    let state_path_str = args.state.clone().unwrap_or_else(|| {
        state::default_state_path(&args.file)
            .to_string_lossy()
            .to_string()
    });
    let state_path = std::path::PathBuf::from(&state_path_str);
    let existing_state = state::load_state(&state_path)?;

    // Reconcile
    let actions = reconciler::reconcile(&template.operations, &existing_state)
        .map_err(|e| -> Box<dyn Error> { e.into() })?;

    if actions.is_empty() {
        out_println!(out, "Nothing to do — template and state are in sync.");
        return Ok(());
    }

    // Execute
    let mut sync_state = existing_state.unwrap_or_else(state::SyncState::new);
    let result = executor::execute_sync(
        &mut template,
        &actions,
        &mut sync_state,
        &state_path,
        client,
        cache_dir,
        out,
        args.prune,
        args.confirm,
    )
    .await?;

    // Print summary
    if out.is_machine_readable() {
        out_println!(out, "{}", serde_json::to_string_pretty(&result)?);
    } else if !out.is_dry_run() {
        let summary = &result.summary;
        if summary.failed == 0 {
            out_println!(
                out,
                "\nSync complete: {}/{} actions succeeded.",
                summary.succeeded,
                summary.total
            );
        } else {
            out_println!(
                out,
                "\nSync complete: {}/{} actions succeeded ({} failed).",
                summary.succeeded,
                summary.total,
                summary.failed
            );
        }
    }

    if result.summary.failed > 0 {
        Err(format!("{} action(s) failed", result.summary.failed).into())
    } else {
        Ok(())
    }
}
