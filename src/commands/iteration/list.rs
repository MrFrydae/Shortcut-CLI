use crate::api;
use crate::out_println;
use crate::output::{OutputConfig, Table};
use std::error::Error;

pub async fn run(
    state: Option<&str>,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let iterations = client
        .list_iterations()
        .send()
        .await
        .map_err(|e| format!("Failed to list iterations: {e}"))?;

    if out.is_quiet() {
        for iter in iterations.iter() {
            if state.is_some_and(|s| !iter.status.eq_ignore_ascii_case(s)) {
                continue;
            }
            out_println!(out, "{}", iter.id);
        }
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "Status", "Dates", "Name"]);
    for iter in iterations.iter() {
        let status = &iter.status;
        if let Some(s) = state
            && !status.eq_ignore_ascii_case(s)
        {
            continue;
        }
        table.add_row(vec![
            iter.id.to_string(),
            status.clone(),
            format!("{} \u{2192} {}", iter.start_date, iter.end_date),
            iter.name.clone(),
        ]);
    }
    out.write_str(format_args!("{}", table.render()))?;
    Ok(())
}
