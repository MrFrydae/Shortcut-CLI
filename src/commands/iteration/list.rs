use std::error::Error;

use crate::api;

pub async fn run(state: Option<&str>, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let iterations = client
        .list_iterations()
        .send()
        .await
        .map_err(|e| format!("Failed to list iterations: {e}"))?;

    for iter in iterations.iter() {
        let status = iter.status.to_string();
        if let Some(filter) = state
            && !status.eq_ignore_ascii_case(filter)
        {
            continue;
        }
        println!(
            "{} - {} ({}, {} \u{2192} {})",
            iter.id, iter.name, status, iter.start_date, iter.end_date
        );
    }
    Ok(())
}
