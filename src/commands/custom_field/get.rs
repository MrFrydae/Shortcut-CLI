use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(id: &str, client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let uuid: uuid::Uuid = id.parse().map_err(|e| format!("Invalid UUID: {e}"))?;

    let field = client
        .get_custom_field()
        .custom_field_public_id(uuid)
        .send()
        .await
        .map_err(|e| format!("Failed to get custom field: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*field)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        out_println!(out, "{}", field.id);
        return Ok(());
    }

    let name: &str = &field.name;
    out_println!(out, "{} - {}", field.id, name);
    out_println!(out, "  Type:        {}", field.field_type);
    out_println!(out, "  Enabled:     {}", field.enabled);
    if let Some(desc) = &field.description {
        let desc: &str = desc;
        out_println!(out, "  Description: {desc}");
    }
    if !field.values.is_empty() {
        out_println!(out, "  Values:");
        for val in &field.values {
            let color = val
                .color_key
                .as_deref()
                .map(|c| format!(" ({c})"))
                .unwrap_or_default();
            let val_name: &str = &val.value;
            out_println!(out, "    {} - {}{}", val.id, val_name, color);
        }
    }

    Ok(())
}
