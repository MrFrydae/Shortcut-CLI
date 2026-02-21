use std::error::Error;

use crate::api;

pub async fn run(id: i64, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a project is irreversible. Pass --confirm to proceed.".into());
    }

    let project = client
        .get_project()
        .project_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get project: {e}"))?;

    let name = project.name.clone();

    client
        .delete_project()
        .project_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete project: {e}"))?;

    println!("Deleted project {id} - {name}");
    Ok(())
}
