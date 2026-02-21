use super::helpers;
use crate::api;
use crate::out_println;
use crate::output::{OutputConfig, Table};
use std::error::Error;
use std::path::Path;

pub async fn run(
    desc: bool,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let labels = client
        .list_labels()
        .send()
        .await
        .map_err(|e| format!("Failed to list labels: {e}"))?;

    if out.is_quiet() {
        for label in labels.iter() {
            out_println!(out, "{}", label.id);
        }
        helpers::update_cache_from_labels(&labels, cache_dir);
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "Color", "Name"]);
    for label in labels.iter() {
        let color = label.color.as_deref().unwrap_or("");
        table.add_row(vec![
            label.id.to_string(),
            color.to_string(),
            label.name.clone(),
        ]);
    }
    out.write_str(format_args!("{}", table.render()))?;

    if desc {
        for label in labels.iter() {
            if let Some(d) = &label.description
                && !d.is_empty()
            {
                out_println!(out, "  {}: {d}", label.id);
            }
        }
    }

    helpers::update_cache_from_labels(&labels, cache_dir);
    Ok(())
}
