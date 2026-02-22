use std::error::Error;

use clap::Args;

use crate::out_println;
use crate::output::OutputConfig;
use crate::stl::{parser, validator};

#[derive(Args)]
pub struct ValidateArgs {
    /// Path to the .sc.yml template file
    pub file: String,
}

pub async fn run(args: &ValidateArgs, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let template = parser::parse_from_path(&args.file)?;

    let errors = validator::validate(&template);

    if errors.is_empty() {
        out_println!(out, "Template is valid.");
        Ok(())
    } else {
        out_println!(out, "Validation errors:");
        for err in &errors {
            out_println!(out, "  - {err}");
        }
        Err(format!("{} validation error(s) found", errors.len()).into())
    }
}
