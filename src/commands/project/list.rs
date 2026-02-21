use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::{OutputConfig, Table};

pub async fn run(
    include_archived: bool,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let projects = client
        .list_projects()
        .send()
        .await
        .map_err(|e| format!("Failed to list projects: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*projects)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        for proj in projects.iter() {
            if !include_archived && proj.archived {
                continue;
            }
            out_println!(out, "{}", proj.id);
        }
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "Stories", "Name"]);
    for proj in projects.iter() {
        if !include_archived && proj.archived {
            continue;
        }
        table.add_row(vec![
            proj.id.to_string(),
            proj.stats.num_stories.to_string(),
            proj.name.clone(),
        ]);
    }
    out.write_str(format_args!("{}", table.render()))?;
    Ok(())
}
