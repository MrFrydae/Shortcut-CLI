use std::collections::HashMap;
use std::path::Path;

use crate::api;

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

pub fn print_threaded_comment(
    comment: &api::types::ThreadedComment,
    cache_dir: &Path,
    indent: usize,
) {
    let prefix = "  ".repeat(indent);
    let author = resolve_member_name(&comment.author_id, cache_dir);
    let text_preview = comment.text.lines().next().unwrap_or("");
    println!(
        "{prefix}#{} {} ({})",
        comment.id, author, comment.created_at
    );
    println!("{prefix}  {text_preview}");
    println!();
    for child in &comment.comments {
        print_threaded_comment(child, cache_dir, indent + 2);
    }
}
