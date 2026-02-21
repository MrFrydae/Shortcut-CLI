use std::collections::HashMap;
use std::error::Error;

use crate::api;

use super::invert_verb;

pub async fn run(story_id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
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
