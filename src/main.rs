use clap::{Parser, Subcommand};
use sc::{api, auth, commands, project};

/// CLI for interacting with Shortcut
#[derive(Parser)]
#[command(name = "sc")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize ~/.sc/ directory for token and cache storage
    Init,
    /// Authenticate with your Shortcut API token
    Login(commands::login::LoginArgs),
    /// Work with epics
    Epic(commands::epic::EpicArgs),
    /// Work with iterations
    Iteration(commands::iteration::IterationArgs),
    /// Work with labels
    Label(commands::label::LabelArgs),
    /// Work with workspace members
    Member(commands::member::MemberArgs),
    /// Work with objectives
    Objective(commands::objective::ObjectiveArgs),
    /// Search across Shortcut entities
    Search(commands::search::SearchArgs),
    /// Work with stories
    Story(commands::story::StoryArgs),
    /// Work with workflows
    Workflow(commands::workflow::WorkflowArgs),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

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
                    Command::Epic(args) => match api::authenticated_client(&store) {
                        Ok(client) => commands::epic::run(&args, &client, root.cache_dir()).await,
                        Err(e) => Err(e.into()),
                    },
                    Command::Iteration(args) => match api::authenticated_client(&store) {
                        Ok(client) => {
                            commands::iteration::run(&args, &client, root.cache_dir()).await
                        }
                        Err(e) => Err(e.into()),
                    },
                    Command::Label(args) => match api::authenticated_client(&store) {
                        Ok(client) => commands::label::run(&args, &client, root.cache_dir()).await,
                        Err(e) => Err(e.into()),
                    },
                    Command::Member(args) => match api::authenticated_client(&store) {
                        Ok(client) => commands::member::run(&args, &client, root.cache_dir()).await,
                        Err(e) => Err(e.into()),
                    },
                    Command::Objective(args) => match api::authenticated_client(&store) {
                        Ok(client) => commands::objective::run(&args, &client).await,
                        Err(e) => Err(e.into()),
                    },
                    Command::Search(args) => match api::authenticated_client(&store) {
                        Ok(client) => commands::search::run(&args, &client).await,
                        Err(e) => Err(e.into()),
                    },
                    Command::Story(args) => match api::authenticated_client(&store) {
                        Ok(client) => commands::story::run(&args, &client, root.cache_dir()).await,
                        Err(e) => Err(e.into()),
                    },
                    Command::Workflow(args) => match api::authenticated_client(&store) {
                        Ok(client) => commands::workflow::run(&args, &client).await,
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
