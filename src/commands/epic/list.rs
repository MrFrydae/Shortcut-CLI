use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::{OutputConfig, Table};

pub async fn run(
    desc: bool,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let mut req = client.list_epics();
    if desc {
        req = req.includes_description(true);
    }
    let epics = req
        .send()
        .await
        .map_err(|e| format!("Failed to list epics: {e}"))?;

    if out.is_json() {
        let json: Vec<serde_json::Value> = epics
            .iter()
            .map(|e| serde_json::json!({"id": e.id, "name": e.name}))
            .collect();
        out_println!(out, "{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }
    if out.is_quiet() {
        for epic in epics.iter() {
            out_println!(out, "{}", epic.id);
        }
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "Name"]);
    for epic in epics.iter() {
        table.add_row(vec![epic.id.to_string(), epic.name.clone()]);
    }
    out.write_str(format_args!("{}", table.render()))?;

    if desc {
        for epic in epics.iter() {
            if let Some(d) = &epic.description {
                out_println!(out, "  {}: {}", epic.id, d);
            }
        }
    }
    Ok(())
}
