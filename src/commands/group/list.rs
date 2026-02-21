use std::error::Error;
use std::path::Path;

use crate::api;

use super::helpers::write_cache;

pub async fn run(
    include_archived: bool,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let groups = client
        .list_groups()
        .send()
        .await
        .map_err(|e| format!("Failed to list groups: {e}"))?;

    for group in groups.iter() {
        if !include_archived && group.archived {
            continue;
        }
        println!(
            "{} - {} (@{}, {} members)",
            group.id,
            group.name,
            group.mention_name.as_str(),
            group.member_ids.len()
        );
    }

    write_cache(&groups, cache_dir);

    Ok(())
}
