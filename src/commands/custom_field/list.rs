use std::error::Error;
use std::path::Path;

use crate::api;
use crate::output::{OutputConfig, Table};

use super::helpers::{build_cache, write_cache};
use crate::out_println;

pub async fn run(
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let fields = client
        .list_custom_fields()
        .send()
        .await
        .map_err(|e| format!("Failed to list custom fields: {e}"))?;

    // Update cache while we have the data
    let cache = build_cache(&fields);
    write_cache(&cache, cache_dir);

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*fields)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if fields.is_empty() {
        out_println!(out, "No custom fields found");
        return Ok(());
    }

    if out.is_quiet() {
        for field in fields.iter() {
            out_println!(out, "{}", field.id);
        }
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "Type", "Name"]);
    for field in fields.iter() {
        let name: &str = &field.name;
        table.add_row(vec![
            field.id.to_string(),
            field.field_type.to_string(),
            name.to_string(),
        ]);
    }
    out.write_str(format_args!("{}", table.render()))?;

    Ok(())
}
