use std::error::Error;

use crate::interactive::{IdChoice, MemberChoice, Prompter, UuidChoice};

use super::create::CreateArgs;

/// Run the interactive wizard, prompting only for fields not already set via CLI flags.
/// Returns a fully populated `CreateArgs` on confirmation, or an error if the user aborts.
pub fn run_wizard(
    base: &CreateArgs,
    prompter: &dyn Prompter,
    members: &[MemberChoice],
    epic_states: &[String],
    group_choices: &[UuidChoice],
    objective_choices: &[IdChoice],
) -> Result<CreateArgs, Box<dyn Error>> {
    let name = if let Some(ref n) = base.name {
        n.clone()
    } else {
        prompter.prompt_text("Epic name")?
    };

    let description = if base.description.is_some() {
        base.description.clone()
    } else {
        prompter.prompt_optional_text("Description")?
    };

    let state = if base.state.is_some() {
        base.state.clone()
    } else {
        let state_refs: Vec<&str> = epic_states.iter().map(|s| s.as_str()).collect();
        prompter.prompt_optional_select("Epic state", &state_refs)?
    };

    let deadline = if base.deadline.is_some() {
        base.deadline.clone()
    } else {
        prompter.prompt_optional_text("Deadline (RFC 3339, e.g. 2025-12-31T00:00:00Z)")?
    };

    let owners = if !base.owners.is_empty() {
        base.owners.clone()
    } else {
        prompter.prompt_multi_select("Owners", members)?
    };

    let labels = if !base.labels.is_empty() {
        base.labels.clone()
    } else {
        prompter.prompt_list("Labels")?
    };

    let group_ids = if !base.group_ids.is_empty() {
        base.group_ids.clone()
    } else {
        prompter
            .prompt_optional_select_uuid("Team", group_choices)?
            .map(|u| vec![u.to_string()])
            .unwrap_or_default()
    };

    let objective_ids = if !base.objective_ids.is_empty() {
        base.objective_ids.clone()
    } else {
        prompter.prompt_multi_select_id("Objectives", objective_choices)?
    };

    let followers = if !base.followers.is_empty() {
        base.followers.clone()
    } else {
        prompter.prompt_multi_select("Followers", members)?
    };

    let requested_by = if base.requested_by.is_some() {
        base.requested_by.clone()
    } else {
        prompter.prompt_optional_text("Requested by (@mention or UUID)")?
    };

    if !prompter.confirm(&format!("Create epic \"{name}\"?"))? {
        return Err("Aborted".into());
    }

    Ok(CreateArgs {
        interactive: false,
        name: Some(name),
        description,
        state,
        deadline,
        owners,
        group_ids,
        labels,
        objective_ids,
        followers,
        requested_by,
    })
}
