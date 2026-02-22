use clap::{Parser, Subcommand};

use crate::commands;

/// CLI for interacting with Shortcut
#[derive(Parser)]
#[command(name = "sc")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output raw JSON instead of human-readable text
    #[arg(long, global = true)]
    pub json: bool,

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
    /// Initialize ~/.sc/ directory for token and cache storage
    Init,
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
