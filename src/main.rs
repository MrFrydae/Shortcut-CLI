use clap::{Parser, Subcommand};

/// CLI for interacting with Shortcut
#[derive(Parser)]
#[command(name = "sc")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {}

fn main() {
    let _cli = Cli::parse();
}
