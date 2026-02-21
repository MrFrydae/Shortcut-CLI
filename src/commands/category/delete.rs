use std::error::Error;

use crate::api;

pub async fn run(id: i64, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
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

    println!("Deleted category {id} - {name}");
    Ok(())
}
