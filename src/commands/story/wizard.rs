use std::error::Error;

use crate::interactive::{IdChoice, MemberChoice, Prompter, UuidChoice};

use super::create::CreateArgs;

pub struct WizardChoices<'a> {
    pub members: &'a [MemberChoice],
    pub workflow_states: &'a [String],
    pub story_types: &'a [&'a str],
    pub epic_choices: &'a [IdChoice],
    pub iteration_choices: &'a [IdChoice],
    pub group_choices: &'a [UuidChoice],
}

/// Run the interactive wizard, prompting only for fields not already set via CLI flags.
/// Returns a fully populated `CreateArgs` on confirmation, or an error if the user aborts.
pub fn run_wizard(
    base: &CreateArgs,
    prompter: &dyn Prompter,
    choices: &WizardChoices<'_>,
) -> Result<CreateArgs, Box<dyn Error>> {
    let name = if let Some(ref n) = base.name {
        n.clone()
    } else {
        prompter.prompt_text("Story name")?
    };

    let description = if base.description.is_some() {
        base.description.clone()
    } else {
        prompter.prompt_optional_text("Description")?
    };

    let story_type = if base.story_type.is_some() {
        base.story_type.clone()
    } else {
        prompter.prompt_optional_select("Story type", choices.story_types)?
    };

    let owner = if !base.owner.is_empty() {
        base.owner.clone()
    } else {
        prompter.prompt_multi_select("Owners", choices.members)?
    };

    let state = if base.state.is_some() {
        base.state.clone()
    } else {
        let state_refs: Vec<&str> = choices.workflow_states.iter().map(|s| s.as_str()).collect();
        prompter.prompt_optional_select("Workflow state", &state_refs)?
    };

    let epic_id = if base.epic_id.is_some() {
        base.epic_id
    } else {
        prompter.prompt_optional_select_id("Epic", choices.epic_choices)?
    };

    let group_id = if base.group_id.is_some() {
        base.group_id.clone()
    } else {
        prompter
            .prompt_optional_select_uuid("Team", choices.group_choices)?
            .map(|u| u.to_string())
    };

    let estimate = if base.estimate.is_some() {
        base.estimate
    } else {
        prompter.prompt_optional_i64("Estimate")?
    };

    let labels = if !base.labels.is_empty() {
        base.labels.clone()
    } else {
        prompter.prompt_list("Labels")?
    };

    let iteration_id = if base.iteration_id.is_some() {
        base.iteration_id
    } else {
        prompter.prompt_optional_select_id("Iteration", choices.iteration_choices)?
    };

    let custom_fields = if !base.custom_fields.is_empty() {
        base.custom_fields.clone()
    } else {
        prompter.prompt_list("Custom fields (FieldName=Value)")?
    };

    if !prompter.confirm(&format!("Create story \"{name}\"?"))? {
        return Err("Aborted".into());
    }

    Ok(CreateArgs {
        interactive: false,
        name: Some(name),
        description,
        story_type,
        owner,
        state,
        epic_id,
        group_id,
        estimate,
        labels,
        iteration_id,
        custom_fields,
    })
}
