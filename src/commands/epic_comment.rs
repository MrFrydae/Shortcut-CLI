use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};

use crate::api;

#[derive(Args)]
pub struct CommentArgs {
    #[command(subcommand)]
    pub action: CommentAction,
}

#[derive(Subcommand)]
pub enum CommentAction {
    /// List all comments on an epic
    List {
        /// The epic ID
        #[arg(long)]
        epic_id: i64,
    },
    /// Add a comment to an epic
    Add {
        /// The epic ID
        #[arg(long)]
        epic_id: i64,
        /// Comment text
        #[arg(long)]
        text: Option<String>,
        /// Read comment text from a file
        #[arg(long)]
        text_file: Option<PathBuf>,
    },
    /// Get a single comment
    Get {
        /// The epic ID
        #[arg(long)]
        epic_id: i64,
        /// The comment ID
        #[arg(long)]
        id: i64,
    },
    /// Update a comment's text
    Update {
        /// The epic ID
        #[arg(long)]
        epic_id: i64,
        /// The comment ID
        #[arg(long)]
        id: i64,
        /// New comment text
        #[arg(long)]
        text: String,
    },
    /// Delete a comment
    Delete {
        /// The epic ID
        #[arg(long)]
        epic_id: i64,
        /// The comment ID
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
}

pub async fn run(
    args: &CommentArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        CommentAction::List { epic_id } => run_list(*epic_id, client, cache_dir).await,
        CommentAction::Add {
            epic_id,
            text,
            text_file,
        } => run_add(*epic_id, text.as_deref(), text_file.as_deref(), client).await,
        CommentAction::Get { epic_id, id } => run_get(*epic_id, *id, client, cache_dir).await,
        CommentAction::Update { epic_id, id, text } => {
            run_update(*epic_id, *id, text, client).await
        }
        CommentAction::Delete {
            epic_id,
            id,
            confirm,
        } => run_delete(*epic_id, *id, *confirm, client).await,
    }
}

async fn run_list(
    epic_id: i64,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let comments = client
        .list_epic_comments()
        .epic_public_id(epic_id)
        .send()
        .await
        .map_err(|e| format!("Failed to list comments: {e}"))?;

    if comments.is_empty() {
        println!("No comments on epic {epic_id}");
        return Ok(());
    }

    println!("Comments on epic {epic_id}:\n");
    for c in comments.iter() {
        print_threaded_comment(c, cache_dir, 1);
    }
    Ok(())
}

fn print_threaded_comment(comment: &api::types::ThreadedComment, cache_dir: &Path, indent: usize) {
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

async fn run_add(
    epic_id: i64,
    text: Option<&str>,
    text_file: Option<&Path>,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    let body = if let Some(path) = text_file {
        std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {e}", path.display()))?
    } else if let Some(t) = text {
        t.to_string()
    } else {
        return Err("Either --text or --text-file is required".into());
    };

    let text_value = body
        .parse::<api::types::CreateEpicCommentText>()
        .map_err(|e| format!("Invalid comment text: {e}"))?;

    let comment = client
        .create_epic_comment()
        .epic_public_id(epic_id)
        .body_map(|b| b.text(text_value))
        .send()
        .await
        .map_err(|e| format!("Failed to create comment: {e}"))?;

    println!("Created comment #{} on epic {epic_id}", comment.id);
    Ok(())
}

async fn run_get(
    epic_id: i64,
    comment_id: i64,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let comment = client
        .get_epic_comment()
        .epic_public_id(epic_id)
        .comment_public_id(comment_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get comment: {e}"))?;

    let author = resolve_member_name(&comment.author_id, cache_dir);

    println!("Comment #{} on epic {epic_id}", comment.id);
    println!("  Author:  {author}");
    println!("  Created: {}", comment.created_at);
    println!("  Updated: {}", comment.updated_at);
    println!();
    println!("  {}", comment.text);
    if !comment.comments.is_empty() {
        println!();
        println!("  Replies:");
        for child in &comment.comments {
            print_threaded_comment(child, cache_dir, 2);
        }
    }
    Ok(())
}

async fn run_update(
    epic_id: i64,
    comment_id: i64,
    text: &str,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    client
        .update_epic_comment()
        .epic_public_id(epic_id)
        .comment_public_id(comment_id)
        .body_map(|b| b.text(text.to_string()))
        .send()
        .await
        .map_err(|e| format!("Failed to update comment: {e}"))?;

    println!("Updated comment #{comment_id} on epic {epic_id}");
    Ok(())
}

async fn run_delete(
    epic_id: i64,
    comment_id: i64,
    confirm: bool,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a comment is irreversible. Pass --confirm to proceed.".into());
    }

    client
        .delete_epic_comment()
        .epic_public_id(epic_id)
        .comment_public_id(comment_id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete comment: {e}"))?;

    println!("Deleted comment #{comment_id} from epic {epic_id}");
    Ok(())
}

/// Best-effort reverse lookup of a member UUID from the member cache.
fn resolve_member_name(uuid: &uuid::Uuid, cache_dir: &Path) -> String {
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
