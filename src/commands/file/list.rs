use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::{OutputConfig, Table, format_template};

use super::helpers::human_size;

pub async fn run(client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let files = client
        .list_files()
        .send()
        .await
        .map_err(|e| format!("Failed to list files: {}", api::format_api_error(&e)))?;

    if out.is_machine_readable() {
        out_println!(out, "{}", serde_json::to_string_pretty(&*files)?);
        return Ok(());
    }
    if let Some(template) = out.format_template() {
        for file in files.iter() {
            out_println!(out, "{}", format_template(template, file)?);
        }
        return Ok(());
    }
    if out.is_quiet() {
        for file in files.iter() {
            out_println!(out, "{}", file.id);
        }
        return Ok(());
    }
    if files.is_empty() {
        out_println!(out, "No files found");
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "Name", "Type", "Size", "Stories"]);
    for file in files.iter() {
        table.add_row(vec![
            file.id.to_string(),
            file.name.clone(),
            file.content_type.clone(),
            human_size(u64::try_from(file.size).unwrap_or(0)),
            file.story_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        ]);
    }
    out.write_str(format_args!("{}", table.render()))?;
    Ok(())
}
