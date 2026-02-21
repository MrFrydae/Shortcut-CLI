use std::error::Error;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

pub async fn run(id: &str, client: &api::Client, out: &OutputConfig) -> Result<(), Box<dyn Error>> {
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e| format!("Invalid entity template UUID: {e}"))?;

    let template = client
        .get_entity_template()
        .entity_template_public_id(uuid)
        .send()
        .await
        .map_err(|e| format!("Failed to get entity template: {e}"))?;

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*template)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        out_println!(out, "{}", template.id);
        return Ok(());
    }

    out_println!(out, "{} - {}", template.id, template.name);
    out_println!(out, "  Created:    {}", template.created_at);
    out_println!(out, "  Updated:    {}", template.updated_at);
    out_println!(out, "  Last used:  {}", template.last_used_at);

    let sc = &template.story_contents;

    if let Some(story_type) = &sc.story_type {
        out_println!(out, "  Story type: {story_type}");
    }
    if let Some(estimate) = sc.estimate {
        out_println!(out, "  Estimate:   {estimate}");
    }
    if let Some(state_id) = sc.workflow_state_id {
        out_println!(out, "  State ID:   {state_id}");
    }
    if let Some(epic_id) = sc.epic_id {
        out_println!(out, "  Epic ID:    {epic_id}");
    }
    if let Some(iter_id) = sc.iteration_id {
        out_println!(out, "  Iteration:  {iter_id}");
    }
    if !sc.labels.is_empty() {
        let names: Vec<&str> = sc.labels.iter().map(|l| l.name.as_str()).collect();
        out_println!(out, "  Labels:     {}", names.join(", "));
    }
    if !sc.owner_ids.is_empty() {
        let ids: Vec<String> = sc.owner_ids.iter().map(|id| id.to_string()).collect();
        out_println!(out, "  Owners:     {}", ids.join(", "));
    }
    if !sc.tasks.is_empty() {
        out_println!(out, "  Tasks:");
        for task in &sc.tasks {
            let check = if task.complete.unwrap_or(false) {
                "x"
            } else {
                " "
            };
            out_println!(out, "    [{}] {}", check, task.description);
        }
    }
    if let Some(desc) = &sc.description
        && !desc.is_empty()
    {
        out_println!(out, "");
        out_println!(out, "{desc}");
    }

    Ok(())
}
