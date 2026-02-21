use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use crate::api;

use super::helpers::resolve_group_id;

pub async fn run(id: &str, client: &api::Client, cache_dir: &Path) -> Result<(), Box<dyn Error>> {
    let group_id = resolve_group_id(id, client, cache_dir).await?;

    let group = client
        .get_group()
        .group_public_id(group_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get group: {e}"))?;

    println!(
        "{} - {} (@{})",
        group.id,
        group.name,
        group.mention_name.as_str()
    );
    println!("  Description: {}", group.description);
    println!("  Archived:    {}", group.archived);
    println!(
        "  Color:       {}",
        group.color.as_deref().unwrap_or("none")
    );
    println!("  Members:     {}", group.member_ids.len());

    if !group.member_ids.is_empty() {
        let members = client
            .list_members()
            .send()
            .await
            .map_err(|e| format!("Failed to list members: {e}"))?;

        let member_map: HashMap<uuid::Uuid, _> = members.iter().map(|m| (m.id, m)).collect();

        for member_id in &group.member_ids {
            if let Some(m) = member_map.get(member_id) {
                let name = m.profile.name.as_deref().unwrap_or("");
                println!("    @{} - {} ({})", m.profile.mention_name, name, m.role);
            } else {
                println!("    {member_id}");
            }
        }
    }

    println!(
        "  Stories:     {} total, {} started, {} backlog",
        group.num_stories, group.num_stories_started, group.num_stories_backlog
    );
    println!("  Epics:       {} started", group.num_epics_started);

    Ok(())
}
