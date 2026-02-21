use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::{OutputConfig, Table};

pub async fn run(client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let templates = client
        .list_entity_templates()
        .send()
        .await
        .map_err(|e| format!("Failed to list entity templates: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*templates)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        for t in templates.iter() {
            out_println!(out, "{}", t.id);
        }
        return Ok(());
    }

    if templates.is_empty() {
        out_println!(out, "No entity templates found");
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "Name"]);
    for t in templates.iter() {
        table.add_row(vec![t.id.to_string(), t.name.clone()]);
    }
    out.write_str(format_args!("{}", table.render()))?;

    Ok(())
}
