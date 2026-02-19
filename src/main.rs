use clap::{Parser, Subcommand};

mod api;
mod auth;
mod commands;

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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Login(args) => commands::login::run(args).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
