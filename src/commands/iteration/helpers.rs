use std::error::Error;
use std::path::Path;

use crate::api;
use crate::commands::member;

/// Fetch current and future iterations as `IdChoice` items for the story wizard.
pub async fn fetch_iteration_choices(
    client: &api::Client,
) -> Result<Vec<crate::interactive::IdChoice>, Box<dyn Error>> {
    let iterations = client
        .list_iterations()
        .send()
        .await
        .map_err(|e| format!("Failed to list iterations: {e}"))?;
    let mut choices: Vec<crate::interactive::IdChoice> = iterations
        .iter()
        .filter(|i| i.status == "unstarted" || i.status == "started")
        .map(|i| crate::interactive::IdChoice {
            display: format!("{} [{}] (#{})", i.name, i.status, i.id),
            id: i.id,
        })
        .collect();
    choices.sort_by(|a, b| a.display.to_lowercase().cmp(&b.display.to_lowercase()));
    Ok(choices)
}

pub async fn resolve_followers(
    followers: &[String],
    client: &api::Client,
    cache_dir: &Path,
) -> Result<Vec<uuid::Uuid>, Box<dyn Error>> {
    let mut ids = Vec::with_capacity(followers.len());
    for follower in followers {
        let uuid = member::resolve_member_id(follower, client, cache_dir).await?;
        ids.push(uuid);
    }
    Ok(ids)
}
