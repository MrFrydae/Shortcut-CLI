use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::{OutputConfig, format_template};

use super::helpers::{human_size, markdown_for};

pub async fn run(id: i64, client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let file = client
        .get_file()
        .file_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get file: {}", api::format_api_error(&e)))?;

    if out.is_machine_readable() {
        out_println!(out, "{}", serde_json::to_string_pretty(&*file)?);
        return Ok(());
    }
    if let Some(template) = out.format_template() {
        out_println!(out, "{}", format_template(template, &*file)?);
        return Ok(());
    }
    if out.is_quiet() {
        out_println!(out, "{}", file.id);
        return Ok(());
    }

    out_println!(out, "{} - {}", file.id, file.name);
    out_println!(out, "  Filename:    {}", file.filename);
    out_println!(out, "  Type:        {}", file.content_type);
    out_println!(
        out,
        "  Size:        {}",
        human_size(u64::try_from(file.size).unwrap_or(0))
    );
    if let Some(url) = &file.url {
        out_println!(out, "  URL:         {url}");
        out_println!(
            out,
            "  Markdown:    {}",
            markdown_for(&file.name, url, &file.content_type)
        );
    }
    if !file.story_ids.is_empty() {
        out_println!(
            out,
            "  Stories:     {}",
            file.story_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if let Some(desc) = file.description.as_deref().filter(|d| !d.is_empty()) {
        out_println!(out, "  Description: {desc}");
    }
    Ok(())
}
