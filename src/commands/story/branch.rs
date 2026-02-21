use std::error::Error;

use clap::Args;

use crate::api;
use crate::output::OutputConfig;
use crate::{out_print, out_println};

use super::git;

#[derive(Args)]
pub struct BranchArgs {
    /// The ID of the story
    #[arg(long)]
    pub id: i64,
    /// Override the type-based prefix (e.g. "hotfix" instead of "feature")
    #[arg(long)]
    pub prefix: Option<String>,
    /// Create and checkout the branch
    #[arg(long, short = 'c')]
    pub checkout: bool,
}

pub async fn run(
    args: &BranchArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let runner = git::RealGitRunner;
    run_with_git(args, client, out, &runner).await
}

pub async fn run_with_git(
    args: &BranchArgs,
    client: &api::Client,
    out: &OutputConfig,
    git_runner: &dyn git::GitRunner,
) -> Result<(), Box<dyn Error>> {
    let story = client
        .get_story()
        .story_public_id(args.id)
        .send()
        .await
        .map_err(|e| format!("Failed to get story: {e}"))?;

    let branch = git::branch_name(
        &story.story_type,
        story.id,
        &story.name,
        args.prefix.as_deref(),
    );

    if args.checkout {
        git_runner.checkout_new_branch(&branch)?;
        out_println!(out, "Checked out new branch: {branch}");
    } else if out.is_json() {
        let json = serde_json::json!({ "branch": branch });
        out_println!(out, "{}", serde_json::to_string_pretty(&json)?);
    } else if out.is_quiet() {
        out_print!(out, "{branch}");
    } else {
        out_println!(out, "{branch}");
    }

    Ok(())
}
