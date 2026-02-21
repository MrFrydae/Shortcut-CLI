use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

/// Best-effort reverse lookup of a member UUID from the member cache.
pub fn resolve_member_name(uuid: &uuid::Uuid, cache_dir: &Path) -> String {
    let cache_path = cache_dir.join("member_cache.json");
    if let Ok(data) = std::fs::read_to_string(&cache_path)
        && let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&data)
    {
        let uuid_str = uuid.to_string();
        for (mention, id) in &map {
            if id == &uuid_str {
                return format!("@{mention}");
            }
        }
    }
    uuid.to_string()
}

/// Recursively print a threaded comment with indentation.
pub fn print_threaded_comment(
    comment: &api::types::ThreadedComment,
    cache_dir: &Path,
    indent: usize,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let prefix = " ".repeat(indent);
    let author = resolve_member_name(&comment.author_id, cache_dir);
    let text_preview = comment.text.lines().next().unwrap_or("");

    out_println!(
        out,
        "{prefix}#{} {} ({})",
        comment.id,
        author,
        comment.created_at
    );
    out_println!(out, "{prefix}  {text_preview}");
    out_println!(out, "");

    for reply in &comment.comments {
        print_threaded_comment(reply, cache_dir, indent + 2, out)?;
    }
    Ok(())
}
