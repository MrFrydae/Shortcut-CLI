use std::error::Error;

use clap::Args;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

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

fn verb_display(verb: &api::types::CreateStoryLinkVerb) -> &'static str {
    match verb {
        api::types::CreateStoryLinkVerb::Blocks => "blocks",
        api::types::CreateStoryLinkVerb::Duplicates => "duplicates",
        api::types::CreateStoryLinkVerb::RelatesTo => "relates to",
    }
}

pub async fn run(
    args: &CreateLinkArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let (api_verb, swapped) = normalize_verb(&args.verb)?;

    let (subject_id, object_id) = if swapped {
        (args.object, args.subject)
    } else {
        (args.subject, args.object)
    };

    let display = verb_display(&api_verb);

    if out.is_dry_run() {
        let body = serde_json::json!({
            "subject_id": subject_id,
            "object_id": object_id,
            "verb": display,
        });
        return out.dry_run_request("POST", "/api/v3/story-links", Some(&body));
    }

    let link = client
        .create_story_link()
        .body_map(|b| b.subject_id(subject_id).object_id(object_id).verb(api_verb))
        .send()
        .await
        .map_err(|e| format!("Failed to create story link: {e}"))?;

    out_println!(
        out,
        "Linked: {} {display} {} (link {})",
        subject_id,
        object_id,
        link.id
    );
    Ok(())
}
