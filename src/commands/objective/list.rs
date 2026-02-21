use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::{OutputConfig, Table};

pub async fn run(
    include_archived: bool,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let objectives = client
        .list_objectives()
        .send()
        .await
        .map_err(|e| format!("Failed to list objectives: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*objectives)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        for obj in objectives.iter() {
            if !include_archived && obj.archived {
                continue;
            }
            out_println!(out, "{}", obj.id);
        }
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "State", "Name"]);
    for obj in objectives.iter() {
        if !include_archived && obj.archived {
            continue;
        }
        table.add_row(vec![
            obj.id.to_string(),
            obj.state.to_string(),
            obj.name.clone(),
        ]);
    }
    out.write_str(format_args!("{}", table.render()))?;
    Ok(())
}
