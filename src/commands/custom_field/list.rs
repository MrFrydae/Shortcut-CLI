use std::error::Error;
use std::path::Path;

use crate::api;

use super::helpers::{build_cache, write_cache};

pub async fn run(client: &api::Client, cache_dir: &Path) -> Result<(), Box<dyn Error>> {
    let fields = client
        .list_custom_fields()
        .send()
        .await
        .map_err(|e| format!("Failed to list custom fields: {e}"))?;

    if fields.is_empty() {
        println!("No custom fields found");
        return Ok(());
    }

    // Update cache while we have the data
    let cache = build_cache(&fields);
    write_cache(&cache, cache_dir);

    for field in fields.iter() {
        let values: Vec<&str> = field
            .values
            .iter()
            .map(|v| v.value.as_str() as &str)
            .collect();
        let name: &str = &field.name;
        println!("{} - {} ({})", field.id, name, field.field_type);
        if !values.is_empty() {
            println!("  Values: {}", values.join(", "));
        }
    }

    Ok(())
}
