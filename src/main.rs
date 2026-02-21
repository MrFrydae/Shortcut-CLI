use clap::{Parser, Subcommand};
use sc::output::{ColorMode, OutputConfig, OutputMode};
use sc::{api, auth, commands, project};

/// CLI for interacting with Shortcut
#[derive(Parser)]
#[command(name = "sc")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Output raw JSON instead of human-readable text
    #[arg(long, global = true)]
    json: bool,

    /// Suppress output; print only IDs
    #[arg(long, short = 'q', global = true)]
    quiet: bool,

    /// Format output using a template string (e.g. "{id} {name}")
    #[arg(long, global = true)]
    format: Option<String>,

    /// Force colored output
    #[arg(long, global = true)]
    color: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,
}

#[derive(Subcommand)]
enum Command {
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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Determine output mode
    let mode = if cli.json {
        OutputMode::Json
    } else if cli.quiet {
        OutputMode::Quiet
    } else if let Some(fmt) = cli.format.clone() {
        OutputMode::Format(fmt)
    } else {
        OutputMode::Human
    };

    // Determine color mode
    let color_mode = if cli.no_color || std::env::var("NO_COLOR").is_ok() {
        ColorMode::Never
    } else if cli.color {
        ColorMode::Always
    } else {
        ColorMode::Auto
    };

    // Set global colored override
    match &color_mode {
        ColorMode::Always => colored::control::set_override(true),
        ColorMode::Never => colored::control::set_override(false),
        ColorMode::Auto => {
            if !atty::is(atty::Stream::Stdout) {
                colored::control::set_override(false);
            }
        }
    }

    let output = OutputConfig::new(mode, color_mode);

    let result = match cli.command {
        Command::Init => commands::init::run(),
        Command::Login(args) => match project::discover_or_init() {
            Ok(root) => {
                let store = auth::FileTokenStore {
                    path: root.token_path(),
                };
                commands::login::run(&args, api::BASE_URL, &store, || {
                    Ok(rpassword::prompt_password("Shortcut API token: ")?)
                })
                .await
            }
            Err(e) => Err(e.into()),
        },
        command => match project::discover() {
            Ok(root) => {
                let store = auth::FileTokenStore {
                    path: root.token_path(),
                };
                match command {
                    Command::Init | Command::Login(_) => unreachable!(),
                    Command::Category(args) => match api::authenticated_client(&store) {
                        Ok(client) => commands::category::run(&args, &client, &output).await,
                        Err(e) => Err(e.into()),
                    },
                    Command::CustomField(args) => match api::authenticated_client(&store) {
                        Ok(client) => {
                            commands::custom_field::run(&args, &client, root.cache_dir(), &output)
                                .await
                        }
                        Err(e) => Err(e.into()),
                    },
                    Command::Doc(args) => match api::authenticated_client(&store) {
                        Ok(client) => commands::doc::run(&args, &client, &output).await,
                        Err(e) => Err(e.into()),
                    },
                    Command::Epic(args) => match api::authenticated_client(&store) {
                        Ok(client) => {
                            commands::epic::run(&args, &client, root.cache_dir(), &output).await
                        }
                        Err(e) => Err(e.into()),
                    },
                    Command::Group(args) => match api::authenticated_client(&store) {
                        Ok(client) => {
                            commands::group::run(&args, &client, root.cache_dir(), &output).await
                        }
                        Err(e) => Err(e.into()),
                    },
                    Command::Iteration(args) => match api::authenticated_client(&store) {
                        Ok(client) => {
                            commands::iteration::run(&args, &client, root.cache_dir(), &output)
                                .await
                        }
                        Err(e) => Err(e.into()),
                    },
                    Command::Label(args) => match api::authenticated_client(&store) {
                        Ok(client) => {
                            commands::label::run(&args, &client, root.cache_dir(), &output).await
                        }
                        Err(e) => Err(e.into()),
                    },
                    Command::Member(args) => match api::authenticated_client(&store) {
                        Ok(client) => {
                            commands::member::run(&args, &client, root.cache_dir(), &output).await
                        }
                        Err(e) => Err(e.into()),
                    },
                    Command::Objective(args) => match api::authenticated_client(&store) {
                        Ok(client) => commands::objective::run(&args, &client, &output).await,
                        Err(e) => Err(e.into()),
                    },
                    Command::Project(args) => match api::authenticated_client(&store) {
                        Ok(client) => commands::project::run(&args, &client, &output).await,
                        Err(e) => Err(e.into()),
                    },
                    Command::Search(args) => match api::authenticated_client(&store) {
                        Ok(client) => commands::search::run(&args, &client, &output).await,
                        Err(e) => Err(e.into()),
                    },
                    Command::Story(args) => match api::authenticated_client(&store) {
                        Ok(client) => {
                            commands::story::run(&args, &client, root.cache_dir(), &output).await
                        }
                        Err(e) => Err(e.into()),
                    },
                    Command::Template(args) => match api::authenticated_client(&store) {
                        Ok(client) => {
                            commands::template::run(&args, &client, root.cache_dir(), &output).await
                        }
                        Err(e) => Err(e.into()),
                    },
                    Command::Workflow(args) => match api::authenticated_client(&store) {
                        Ok(client) => commands::workflow::run(&args, &client, &output).await,
                        Err(e) => Err(e.into()),
                    },
                }
            }
            Err(e) => Err(e.into()),
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
