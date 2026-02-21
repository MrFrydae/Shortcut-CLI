use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::{OutputConfig, Table};

pub async fn run(client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let docs = client
        .list_docs()
        .send()
        .await
        .map_err(|e| format!("Failed to list documents: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*docs)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        for doc in docs.iter() {
            out_println!(out, "{}", doc.id);
        }
        return Ok(());
    }

    if docs.is_empty() {
        out_println!(out, "No documents found");
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "Name"]);
    for doc in docs.iter() {
        let title = doc.title.as_deref().unwrap_or("(untitled)");
        table.add_row(vec![doc.id.to_string(), title.to_string()]);
    }
    out.write_str(format_args!("{}", table.render()))?;

    Ok(())
}
