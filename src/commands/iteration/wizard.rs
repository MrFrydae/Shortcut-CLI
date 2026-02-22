use std::error::Error;

use crate::interactive::{MemberChoice, Prompter};

use super::create::CreateArgs;

/// Run the interactive wizard, prompting only for fields not already set via CLI flags.
/// Returns a fully populated `CreateArgs` on confirmation, or an error if the user aborts.
pub fn run_wizard(
    base: &CreateArgs,
    prompter: &dyn Prompter,
    members: &[MemberChoice],
) -> Result<CreateArgs, Box<dyn Error>> {
    let name = if let Some(ref n) = base.name {
        n.clone()
    } else {
        prompter.prompt_text("Iteration name")?
    };

    let start_date = if let Some(ref s) = base.start_date {
        s.clone()
    } else {
        prompter.prompt_text("Start date (YYYY-MM-DD)")?
    };

    let end_date = if let Some(ref e) = base.end_date {
        e.clone()
    } else {
        prompter.prompt_text("End date (YYYY-MM-DD)")?
    };

    let description = if base.description.is_some() {
        base.description.clone()
    } else {
        prompter.prompt_optional_text("Description")?
    };

    let followers = if !base.followers.is_empty() {
        base.followers.clone()
    } else {
        prompter.prompt_multi_select("Followers", members)?
    };

    let labels = if !base.labels.is_empty() {
        base.labels.clone()
    } else {
        prompter.prompt_list("Labels")?
    };

    let group_ids = if !base.group_ids.is_empty() {
        base.group_ids.clone()
    } else {
        let ids_str = prompter.prompt_list("Group IDs (UUID)")?;
        ids_str
            .iter()
            .map(|s| {
                s.parse::<uuid::Uuid>()
                    .map_err(|_| format!("Invalid UUID: {s}").into())
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?
    };

    if !prompter.confirm(&format!(
        "Create iteration \"{name}\" ({start_date} \u{2192} {end_date})?"
    ))? {
        return Err("Aborted".into());
    }

    Ok(CreateArgs {
        interactive: false,
        name: Some(name),
        start_date: Some(start_date),
        end_date: Some(end_date),
        description,
        followers,
        labels,
        group_ids,
    })
}
