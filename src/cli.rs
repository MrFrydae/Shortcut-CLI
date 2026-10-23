use clap::{Parser, Subcommand};

use crate::commands;

/// CLI for interacting with Shortcut
#[derive(Parser)]
#[command(name = "shortcut", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output raw JSON instead of human-readable text
    #[arg(long, global = true)]
    pub json: bool,

    /// Output TOON instead of human-readable text
    #[arg(long, global = true)]
    pub toon: bool,

    /// Suppress output; print only IDs
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Format output using a template string (e.g. "{id} {name}")
    #[arg(long, global = true)]
    pub format: Option<String>,

    /// Force colored output
    #[arg(long, global = true)]
    pub color: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Preview the API request without sending it
    #[arg(long, global = true)]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize ~/.shortcut/ directory for token and cache storage
    Init,
    /// Print the current project's hashed directory under ~/.shortcut/projects/
    ///
    /// Uses the same ancestor and linked Git worktree discovery as other commands.
    /// Prints only the absolute directory path by default, suitable for shell substitution.
    /// The saved token is in the `token` file inside this directory.
    /// No token or network access is required; no files are read or created inside it.
    /// Fails if no project is registered for the current directory.
    ///
    /// JSON/TOON and --format expose config_dir, token_path, and cache_dir.
    #[command(
        after_help = "Examples:\n  shortcut config-dir\n  ls -la \"$(shortcut config-dir)\"\n  shortcut config-dir --format '{token_path}'\n  shortcut config-dir --json"
    )]
    ConfigDir,
    /// Authenticate with your Shortcut API token
    Login(commands::login::LoginArgs),
    /// Work with categories
    Category(commands::category::CategoryArgs),
    /// Work with custom fields
    CustomField(commands::custom_field::CustomFieldArgs),
    /// Work with documents
    Doc(commands::doc::DocArgs),
    /// Work with epics
    Epic(commands::epic::EpicArgs),
    /// Work with groups
    Group(commands::group::GroupArgs),
    /// Work with iterations
    Iteration(commands::iteration::IterationArgs),
    /// Work with labels
    Label(commands::label::LabelArgs),
    /// Work with workspace members
    Member(commands::member::MemberArgs),
    /// Work with objectives
    Objective(commands::objective::ObjectiveArgs),
    /// Work with projects
    Project(commands::project::ProjectArgs),
    /// Search across Shortcut entities
    Search(commands::search::SearchArgs),
    /// Work with stories
    Story(commands::story::StoryArgs),
    /// Work with entity templates
    Template(commands::template::TemplateArgs),
    /// Work with workflows
    Workflow(commands::workflow::WorkflowArgs),
    /// Generate shell completion scripts
    #[command(hide = true)]
    Completions {
        /// The shell to generate completions for
        shell: clap_complete::Shell,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_toon_global_flag() {
        let cli = Cli::parse_from(["shortcut", "--toon", "member", "--list"]);
        assert!(cli.toon);
        assert!(!cli.json);
    }

    #[test]
    fn parses_json_and_toon_flags_together() {
        let cli = Cli::parse_from(["shortcut", "--json", "--toon", "member", "--list"]);
        assert!(cli.json);
        assert!(cli.toon);
    }
}
