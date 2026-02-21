use std::error::Error;

use clap::Args;

use crate::out_println;
use crate::output::OutputConfig;

use super::git;

#[derive(Args)]
pub struct CommitArgs {
    /// Explicit story ID (overrides branch detection)
    #[arg(long)]
    pub id: Option<i64>,
    /// Commit message
    #[arg(short, long)]
    pub message: String,
    /// Additional arguments to pass to git commit (after --)
    #[arg(last = true)]
    pub extra_args: Vec<String>,
}

pub fn run(args: &CommitArgs, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let runner = git::RealGitRunner;
    run_with_git(args, out, &runner)
}

pub fn run_with_git(
    args: &CommitArgs,
    out: &OutputConfig,
    git_runner: &dyn git::GitRunner,
) -> Result<(), Box<dyn Error>> {
    let story_id = match args.id {
        Some(id) => id,
        None => {
            let branch = git_runner.current_branch()?;
            git::extract_story_id_from_branch(&branch).ok_or_else(|| {
                format!(
                    "Could not detect story ID from branch '{branch}'. Use --id to specify explicitly."
                )
            })?
        }
    };

    let prefixed_message = format!("[sc-{story_id}] {}", args.message);
    let mut commit_args: Vec<&str> = vec!["-m", &prefixed_message];
    for extra in &args.extra_args {
        commit_args.push(extra.as_str());
    }

    let output = git_runner.commit(&commit_args)?;
    if !output.is_empty() {
        out_println!(out, "{output}");
    }

    Ok(())
}
