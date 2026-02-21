use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(
    id: i64,
    confirm: bool,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if out.is_dry_run() {
        return out.dry_run_request::<serde_json::Value>(
            "DELETE",
            &format!("/api/v3/categories/{id}"),
            None,
        );
    }

    if !confirm {
        return Err("Deleting a category is irreversible. Pass --confirm to proceed.".into());
    }

    let category = client
        .get_category()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get category: {e}"))?;

    let name = category.name.clone();

    client
        .delete_category()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete category: {e}"))?;

    if out.is_quiet() {
        return Ok(());
    }

    out_println!(out, "Deleted category {id} - {name}");
    Ok(())
}
