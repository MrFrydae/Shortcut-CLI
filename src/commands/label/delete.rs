use std::error::Error;

use crate::api;

pub async fn run(id: i64, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a label is irreversible. Pass --confirm to proceed.".into());
    }

    let label = client
        .get_label()
        .label_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get label: {e}"))?;

    let name = label.name.clone();

    client
        .delete_label()
        .label_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete label: {e}"))?;

    println!("Deleted label {id} - {name}");
    Ok(())
}
