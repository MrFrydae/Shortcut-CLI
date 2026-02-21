use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::{OutputConfig, Table};

pub async fn run(
    include_archived: bool,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let categories = client
        .list_categories()
        .send()
        .await
        .map_err(|e| format!("Failed to list categories: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*categories)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        for cat in categories.iter() {
            if !include_archived && cat.archived {
                continue;
            }
            out_println!(out, "{}", cat.id);
        }
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "Name"]);
    for cat in categories.iter() {
        if !include_archived && cat.archived {
            continue;
        }
        table.add_row(vec![cat.id.to_string(), cat.name.clone()]);
    }
    out.write_str(format_args!("{}", table.render()))?;
    Ok(())
}
