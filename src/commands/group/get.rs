use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use crate::api;
use crate::output::OutputConfig;

use super::helpers::resolve_group_id;
use crate::out_println;

pub async fn run(
    id: &str,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let group_id = resolve_group_id(id, client, cache_dir).await?;

    let group = client
        .get_group()
        .group_public_id(group_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get group: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*group)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        out_println!(out, "{}", group.id);
        return Ok(());
    }

    out_println!(
        out,
        "{} - {} (@{})",
        group.id,
        group.name,
        group.mention_name.as_str()
    );
    out_println!(out, "  Description: {}", group.description);
    out_println!(out, "  Archived:    {}", group.archived);
    out_println!(
        out,
        "  Color:       {}",
        group.color.as_deref().unwrap_or("none")
    );
    out_println!(out, "  Members:     {}", group.member_ids.len());

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
                out_println!(
                    out,
                    "    @{} - {} ({})",
                    m.profile.mention_name,
                    name,
                    m.role
                );
            } else {
                out_println!(out, "    {member_id}");
            }
        }
    }

    out_println!(
        out,
        "  Stories:     {} total, {} started, {} backlog",
        group.num_stories,
        group.num_stories_started,
        group.num_stories_backlog
    );
    out_println!(out, "  Epics:       {} started", group.num_epics_started);

    Ok(())
}
