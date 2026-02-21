use std::error::Error;

use crate::api;

pub async fn run(id: &str, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let uuid: uuid::Uuid = id.parse().map_err(|e| format!("Invalid UUID: {e}"))?;

    let field = client
        .get_custom_field()
        .custom_field_public_id(uuid)
        .send()
        .await
        .map_err(|e| format!("Failed to get custom field: {e}"))?;

    let name: &str = &field.name;
    println!("{} - {}", field.id, name);
    println!("  Type:        {}", field.field_type);
    println!("  Enabled:     {}", field.enabled);
    if let Some(desc) = &field.description {
        let desc: &str = desc;
        println!("  Description: {desc}");
    }
    if !field.values.is_empty() {
        println!("  Values:");
        for val in &field.values {
            let color = val
                .color_key
                .as_deref()
                .map(|c| format!(" ({c})"))
                .unwrap_or_default();
            let val_name: &str = &val.value;
            println!("    {} - {}{}", val.id, val_name, color);
        }
    }

    Ok(())
}
