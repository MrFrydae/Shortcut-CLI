use std::error::Error;

use crate::api;

pub async fn run(desc: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let mut req = client.list_epics();
    if desc {
        req = req.includes_description(true);
    }
    let epics = req
        .send()
        .await
        .map_err(|e| format!("Failed to list epics: {e}"))?;
    for epic in epics.iter() {
        println!("{} - {}", epic.id, epic.name);
        if desc && let Some(d) = &epic.description {
            println!("  {}", d);
        }
    }
    Ok(())
}
