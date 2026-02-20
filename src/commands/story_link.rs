use std::collections::HashMap;
use std::error::Error;

use clap::{Args, Subcommand};

use crate::api;

#[derive(Args)]
pub struct LinkArgs {
    #[command(subcommand)]
    pub action: LinkAction,
}

#[derive(Subcommand)]
pub enum LinkAction {
    /// Create a relationship between two stories
    Create(CreateLinkArgs),
    /// List all links on a story
    List {
        /// The story ID
        #[arg(long)]
        story_id: i64,
    },
    /// Delete a story link
    Delete {
        /// The story link ID
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateLinkArgs {
    /// The subject story ID (the story that performs the action)
    #[arg(long)]
    pub subject: i64,
    /// The object story ID (the story that receives the action)
    #[arg(long)]
    pub object: i64,
    /// The relationship verb: blocks, blocked-by, duplicates, relates-to
    #[arg(long)]
    pub verb: String,
}

pub async fn run(args: &LinkArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    match &args.action {
        LinkAction::Create(create_args) => run_create(create_args, client).await,
        LinkAction::List { story_id } => run_list(*story_id, client).await,
        LinkAction::Delete { id, confirm } => run_delete(*id, *confirm, client).await,
    }
}

/// Normalize a user-provided verb string into the canonical API verb,
/// returning `(api_verb, swapped)` where `swapped` is true if subject/object
/// should be swapped (for "blocked-by" / "is blocked by").
fn normalize_verb(verb: &str) -> Result<(api::types::CreateStoryLinkVerb, bool), String> {
    match verb.to_lowercase().trim() {
        "blocks" => Ok((api::types::CreateStoryLinkVerb::Blocks, false)),
        "blocked-by" | "is blocked by" => Ok((api::types::CreateStoryLinkVerb::Blocks, true)),
        "duplicates" => Ok((api::types::CreateStoryLinkVerb::Duplicates, false)),
        "relates" | "relates-to" | "relates to" => {
            Ok((api::types::CreateStoryLinkVerb::RelatesTo, false))
        }
        _ => Err(format!(
            "Unknown verb '{verb}'. Valid verbs: blocks, blocked-by, duplicates, relates-to"
        )),
    }
}

/// Return the display string for a verb enum value.
fn verb_display(verb: &api::types::CreateStoryLinkVerb) -> &'static str {
    match verb {
        api::types::CreateStoryLinkVerb::Blocks => "blocks",
        api::types::CreateStoryLinkVerb::Duplicates => "duplicates",
        api::types::CreateStoryLinkVerb::RelatesTo => "relates to",
    }
}

/// Invert a verb string for displaying the object's perspective.
pub fn invert_verb(verb: &str) -> &'static str {
    match verb {
        "blocks" => "blocked by",
        "duplicates" => "duplicated by",
        "relates to" => "relates to",
        _ => "linked to",
    }
}

async fn run_create(args: &CreateLinkArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let (api_verb, swapped) = normalize_verb(&args.verb)?;

    let (subject_id, object_id) = if swapped {
        (args.object, args.subject)
    } else {
        (args.subject, args.object)
    };

    let display = verb_display(&api_verb);

    let link = client
        .create_story_link()
        .body_map(|b| b.subject_id(subject_id).object_id(object_id).verb(api_verb))
        .send()
        .await
        .map_err(|e| format!("Failed to create story link: {e}"))?;

    println!(
        "Linked: {} {display} {} (link {})",
        subject_id, object_id, link.id
    );
    Ok(())
}

async fn run_list(story_id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let story = client
        .get_story()
        .story_public_id(story_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get story: {e}"))?;

    if story.story_links.is_empty() {
        println!("No links on story {story_id}");
        return Ok(());
    }

    // Collect unique other-story IDs to fetch names
    let mut other_ids: Vec<i64> = Vec::new();
    for link in &story.story_links {
        let other_id = if link.type_ == "subject" {
            link.object_id
        } else {
            link.subject_id
        };
        if !other_ids.contains(&other_id) {
            other_ids.push(other_id);
        }
    }

    // Fetch names for linked stories
    let mut names: HashMap<i64, String> = HashMap::new();
    for id in &other_ids {
        match client.get_story().story_public_id(*id).send().await {
            Ok(s) => {
                names.insert(*id, s.name.clone());
            }
            Err(_) => {
                names.insert(*id, "(unknown)".to_string());
            }
        }
    }

    for link in &story.story_links {
        let (display_verb, other_id) = if link.type_ == "subject" {
            (link.verb.as_str(), link.object_id)
        } else {
            (invert_verb(&link.verb), link.subject_id)
        };
        let name = names.get(&other_id).map(|s| s.as_str()).unwrap_or("?");
        println!("  {}: {display_verb} {other_id} - {name}", link.id);
    }
    Ok(())
}

async fn run_delete(id: i64, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a story link is irreversible. Pass --confirm to proceed.".into());
    }

    let link = client
        .get_story_link()
        .story_link_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get story link: {e}"))?;

    client
        .delete_story_link()
        .story_link_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete story link: {e}"))?;

    println!(
        "Deleted story link {} ({} {} {})",
        link.id, link.subject_id, link.verb, link.object_id
    );
    Ok(())
}
