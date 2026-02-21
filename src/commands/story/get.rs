use std::error::Error;
use std::path::Path;

use crate::api;

use super::super::custom_field;
use super::link::invert_verb;

pub async fn run(id: i64, client: &api::Client, cache_dir: &Path) -> Result<(), Box<dyn Error>> {
    let story = client
        .get_story()
        .story_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get story: {e}"))?;

    println!("{} - {}", story.id, story.name);
    println!("  Type:        {}", story.story_type);
    println!("  State ID:    {}", story.workflow_state_id);
    println!("  Workflow ID: {}", story.workflow_id);
    if let Some(epic_id) = story.epic_id {
        println!("  Epic ID:     {epic_id}");
    }
    if let Some(estimate) = story.estimate {
        println!("  Estimate:    {estimate}");
    }
    if !story.owner_ids.is_empty() {
        let ids: Vec<String> = story.owner_ids.iter().map(|id| id.to_string()).collect();
        println!("  Owners:      {}", ids.join(", "));
    }
    if !story.labels.is_empty() {
        let names: Vec<&str> = story.labels.iter().map(|l| l.name.as_str()).collect();
        println!("  Labels:      {}", names.join(", "));
    }
    if !story.description.is_empty() {
        println!("  Description: {}", story.description);
    }
    if !story.story_links.is_empty() {
        println!("  Links:");
        for link in &story.story_links {
            let (display_verb, other_id) = if link.type_ == "subject" {
                (link.verb.as_str(), link.object_id)
            } else {
                (invert_verb(&link.verb), link.subject_id)
            };
            println!("    {display_verb} {other_id} (link {})", link.id);
        }
    }
    if !story.custom_fields.is_empty() {
        let field_ids: Vec<uuid::Uuid> = story.custom_fields.iter().map(|cf| cf.field_id).collect();
        let names = custom_field::resolve_custom_field_names(&field_ids, client, cache_dir).await?;
        for cf in &story.custom_fields {
            let field_name = names
                .get(&cf.field_id.to_string())
                .map(|s| s.as_str())
                .unwrap_or("Unknown");
            println!("  {}: {}", field_name, cf.value);
        }
    }

    Ok(())
}
