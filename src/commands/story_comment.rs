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
    /// List all comments on a story
    List {
        /// The story ID
        #[arg(long)]
        story_id: i64,
    },
    /// Add a comment to a story
    Add {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// Comment text
        #[arg(long)]
        text: Option<String>,
        /// Read comment text from a file
        #[arg(long)]
        text_file: Option<PathBuf>,
    },
    /// Get a single comment
    Get {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The comment ID
        #[arg(long)]
        id: i64,
    },
    /// Update a comment's text
    Update {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The comment ID
        #[arg(long)]
        id: i64,
        /// New comment text
        #[arg(long)]
        text: String,
    },
    /// Delete a comment
    Delete {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The comment ID
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// Add a reaction to a comment
    React {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The comment ID
        #[arg(long)]
        comment_id: i64,
        /// The emoji name (e.g. thumbsup)
        #[arg(long)]
        emoji: String,
    },
    /// Remove a reaction from a comment
    Unreact {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The comment ID
        #[arg(long)]
        comment_id: i64,
        /// The emoji name (e.g. thumbsup)
        #[arg(long)]
        emoji: String,
    },
}

pub async fn run(
    args: &CommentArgs,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    match &args.action {
        CommentAction::List { story_id } => run_list(*story_id, client, cache_dir).await,
        CommentAction::Add {
            story_id,
            text,
            text_file,
        } => run_add(*story_id, text.as_deref(), text_file.as_deref(), client).await,
        CommentAction::Get { story_id, id } => run_get(*story_id, *id, client, cache_dir).await,
        CommentAction::Update { story_id, id, text } => {
            run_update(*story_id, *id, text, client).await
        }
        CommentAction::Delete {
            story_id,
            id,
            confirm,
        } => run_delete(*story_id, *id, *confirm, client).await,
        CommentAction::React {
            story_id,
            comment_id,
            emoji,
        } => run_react(*story_id, *comment_id, emoji, client).await,
        CommentAction::Unreact {
            story_id,
            comment_id,
            emoji,
        } => run_unreact(*story_id, *comment_id, emoji, client).await,
    }
}

async fn run_list(
    story_id: i64,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let comments = client
        .list_story_comment()
        .story_public_id(story_id)
        .send()
        .await
        .map_err(|e| format!("Failed to list comments: {e}"))?;

    if comments.is_empty() {
        println!("No comments on story {story_id}");
        return Ok(());
    }

    println!("Comments on story {story_id}:\n");
    for c in comments.iter() {
        let author = match &c.author_id {
            Some(uuid) => resolve_member_name(uuid, cache_dir),
            None => "unknown".to_string(),
        };
        let text_preview = c.text.as_deref().unwrap_or("").lines().next().unwrap_or("");
        println!("  #{} {} ({})", c.id, author, c.created_at);
        println!("    {text_preview}");
        println!();
    }
    Ok(())
}

async fn run_add(
    story_id: i64,
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
        .parse::<api::types::CreateStoryCommentText>()
        .map_err(|e| format!("Invalid comment text: {e}"))?;

    let comment = client
        .create_story_comment()
        .story_public_id(story_id)
        .body_map(|b| b.text(text_value))
        .send()
        .await
        .map_err(|e| format!("Failed to create comment: {e}"))?;

    println!("Created comment #{} on story {story_id}", comment.id);
    Ok(())
}

async fn run_get(
    story_id: i64,
    comment_id: i64,
    client: &api::Client,
    cache_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let comment = client
        .get_story_comment()
        .story_public_id(story_id)
        .comment_public_id(comment_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get comment: {e}"))?;

    let author = match &comment.author_id {
        Some(uuid) => resolve_member_name(uuid, cache_dir),
        None => "unknown".to_string(),
    };

    println!("Comment #{} on story {story_id}", comment.id);
    println!("  Author:  {author}");
    println!("  Created: {}", comment.created_at);
    if let Some(updated) = &comment.updated_at {
        println!("  Updated: {updated}");
    }
    println!();
    if let Some(text) = &comment.text {
        println!("  {text}");
    }
    if !comment.reactions.is_empty() {
        println!();
        let reactions: Vec<String> = comment
            .reactions
            .iter()
            .map(|r| {
                let name = r.emoji.trim_matches(':');
                format!("{name} ({})", r.permission_ids.len())
            })
            .collect();
        println!("  Reactions: {}", reactions.join(", "));
    }
    Ok(())
}

async fn run_update(
    story_id: i64,
    comment_id: i64,
    text: &str,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    let text_value = text
        .parse::<api::types::UpdateStoryCommentText>()
        .map_err(|e| format!("Invalid comment text: {e}"))?;

    client
        .update_story_comment()
        .story_public_id(story_id)
        .comment_public_id(comment_id)
        .body_map(|b| b.text(text_value))
        .send()
        .await
        .map_err(|e| format!("Failed to update comment: {e}"))?;

    println!("Updated comment #{comment_id} on story {story_id}");
    Ok(())
}

async fn run_delete(
    story_id: i64,
    comment_id: i64,
    confirm: bool,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a comment is irreversible. Pass --confirm to proceed.".into());
    }

    client
        .delete_story_comment()
        .story_public_id(story_id)
        .comment_public_id(comment_id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete comment: {e}"))?;

    println!("Deleted comment #{comment_id} from story {story_id}");
    Ok(())
}

fn format_emoji(emoji: &str) -> String {
    if emoji.starts_with(':') && emoji.ends_with(':') {
        emoji.to_string()
    } else {
        format!(":{emoji}:")
    }
}

async fn run_react(
    story_id: i64,
    comment_id: i64,
    emoji: &str,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    let emoji_str = format_emoji(emoji);

    client
        .create_story_reaction()
        .story_public_id(story_id)
        .comment_public_id(comment_id)
        .body_map(|b| b.emoji(emoji_str.clone()))
        .send()
        .await
        .map_err(|e| format!("Failed to add reaction: {e}"))?;

    println!("Added {emoji_str} to comment #{comment_id} on story {story_id}");
    Ok(())
}

async fn run_unreact(
    story_id: i64,
    comment_id: i64,
    emoji: &str,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    let emoji_str = format_emoji(emoji);

    client
        .delete_story_reaction()
        .story_public_id(story_id)
        .comment_public_id(comment_id)
        .body_map(|b| b.emoji(emoji_str.clone()))
        .send()
        .await
        .map_err(|e| format!("Failed to remove reaction: {e}"))?;

    println!("Removed {emoji_str} from comment #{comment_id} on story {story_id}");
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
