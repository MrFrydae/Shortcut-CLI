use std::error::Error;
use std::io::Write;

use clap::{Command as ClapCommand, CommandFactory};
use clap_complete::{Shell, generate};

use crate::cli::Cli;

/// Build a command tree with only visible subcommands, so hidden commands
/// like `completions` itself don't appear in the generated completion scripts.
fn build_visible_cmd() -> ClapCommand {
    let base = Cli::command();
    let mut cmd = ClapCommand::new("sc");

    for arg in base.get_arguments() {
        cmd = cmd.arg(arg.clone());
    }

    for sub in base.get_subcommands() {
        if !sub.is_hide_set() {
            cmd = cmd.subcommand(sub.clone());
        }
    }

    cmd
}

pub fn run(shell: Shell, writer: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    let mut cmd = build_visible_cmd();
    generate(shell, &mut cmd, "sc", writer);
    Ok(())
}
