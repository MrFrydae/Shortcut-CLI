use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use crate::api;

use super::helpers::{normalize_name, write_cache};

pub async fn run(desc: bool, client: &api::Client, cache_dir: &Path) -> Result<(), Box<dyn Error>> {
    let labels = client
        .list_labels()
        .slim(false)
        .send()
        .await
        .map_err(|e| format!("Failed to list labels: {e}"))?;

    for label in labels.iter() {
        let color = label
            .color
            .as_deref()
            .map(|c| format!(" ({c})"))
            .unwrap_or_default();
        println!("{} - {}{}", label.id, label.name, color);
        if desc && let Some(d) = &label.description {
            println!("  {d}");
        }
    }

    // Update label cache
    let map: HashMap<String, i64> = labels
        .iter()
        .map(|l| (normalize_name(&l.name), l.id))
        .collect();
    write_cache(&map, cache_dir);

    Ok(())
}
