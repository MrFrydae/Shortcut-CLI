use clap::{Parser, Subcommand};

mod api;

/// CLI for interacting with Shortcut
#[derive(Parser)]
#[command(name = "sc")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {}

#[tokio::main]
async fn main() {
    let _cli = Cli::parse();
}
