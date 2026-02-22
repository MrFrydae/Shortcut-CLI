use std::error::Error;

use crate::api;

/// Fetch non-archived objectives as `IdChoice` items for the epic wizard.
pub async fn fetch_objective_choices(
    client: &api::Client,
) -> Result<Vec<crate::interactive::IdChoice>, Box<dyn Error>> {
    let objectives = client
        .list_objectives()
        .send()
        .await
        .map_err(|e| format!("Failed to list objectives: {e}"))?;
    let mut choices: Vec<crate::interactive::IdChoice> = objectives
        .iter()
        .filter(|o| !o.archived)
        .map(|o| crate::interactive::IdChoice {
            display: format!("{} [{}] (#{})", o.name, o.state, o.id),
            id: o.id,
        })
        .collect();
    choices.sort_by(|a, b| a.display.to_lowercase().cmp(&b.display.to_lowercase()));
    Ok(choices)
}

pub fn build_categories(names: &[String]) -> Result<Vec<api::types::CreateCategoryParams>, String> {
    names
        .iter()
        .map(|n| {
            Ok(api::types::CreateCategoryParams {
                name: n
                    .parse()
                    .map_err(|e| format!("Invalid category name: {e}"))?,
                color: None,
                external_id: None,
            })
        })
        .collect()
}
