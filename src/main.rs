use clap::{Parser, Subcommand};
use sc::{api, auth, commands};

/// CLI for interacting with Shortcut
#[derive(Parser)]
#[command(name = "sc")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Authenticate with your Shortcut API token
    Login(commands::login::LoginArgs),
    /// Work with epics
    Epic(commands::epic::EpicArgs),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Login(args) => {
            commands::login::run(&args, api::BASE_URL, &auth::KeychainStore, || {
                Ok(rpassword::prompt_password("Shortcut API token: ")?)
            })
            .await
        }
        Command::Epic(args) => match api::authenticated_client() {
            Ok(client) => commands::epic::run(&args, &client).await,
            Err(e) => Err(e.into()),
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
