mod support;

#[path = "iteration/wizard.rs"]
mod wizard;

pub fn make_create_args(
    name: &str,
    start_date: &str,
    end_date: &str,
) -> sc::commands::iteration::CreateArgs {
    sc::commands::iteration::CreateArgs {
        interactive: false,
        name: Some(name.to_string()),
        start_date: Some(start_date.to_string()),
        end_date: Some(end_date.to_string()),
        description: None,
        followers: vec![],
        labels: vec![],
        group_ids: vec![],
    }
}
