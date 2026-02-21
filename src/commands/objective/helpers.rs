use crate::api;

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
