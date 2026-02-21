use std::error::Error;
use std::path::Path;

use crate::api;
use crate::commands::member;

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
