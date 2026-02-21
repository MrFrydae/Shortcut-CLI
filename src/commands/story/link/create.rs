use std::error::Error;

use clap::Args;

use crate::api;

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

pub async fn run(args: &CreateLinkArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
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
