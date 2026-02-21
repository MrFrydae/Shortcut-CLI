use std::error::Error;
use std::path::Path;

use crate::api;
use crate::output::{OutputConfig, Table};

use super::helpers::write_cache;
use crate::out_println;

pub async fn run(
    include_archived: bool,
    client: &api::Client,
    cache_dir: &Path,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let groups = client
        .list_groups()
        .send()
        .await
        .map_err(|e| format!("Failed to list groups: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*groups)?;
        out.write_str(format_args!("{json}"))?;
        write_cache(&groups, cache_dir);
        return Ok(());
    }

    if out.is_quiet() {
        for group in groups.iter() {
            if !include_archived && group.archived {
                continue;
            }
            out_println!(out, "{}", group.id);
        }
        write_cache(&groups, cache_dir);
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "Members", "Name"]);
    for group in groups.iter() {
        if !include_archived && group.archived {
            continue;
        }
        table.add_row(vec![
            group.id.to_string(),
            group.member_ids.len().to_string(),
            format!("{} (@{})", group.name, group.mention_name.as_str()),
        ]);
    }
    out.write_str(format_args!("{}", table.render()))?;

    write_cache(&groups, cache_dir);

    Ok(())
}
