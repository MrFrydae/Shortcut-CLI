use sc::commands::iteration::wizard::run_wizard;
use sc::interactive::{MemberChoice, MockAnswer, MockPrompter};

use crate::make_create_args;

fn test_members() -> Vec<MemberChoice> {
    vec![
        MemberChoice {
            display: "Alice Smith (@alice)".into(),
            value: "@alice".into(),
        },
        MemberChoice {
            display: "Bob Jones (@bob)".into(),
            value: "@bob".into(),
        },
    ]
}

/// Helper to build a default CreateArgs with interactive=true and no required fields.
fn empty_interactive_args() -> sc::commands::iteration::CreateArgs {
    sc::commands::iteration::CreateArgs {
        interactive: true,
        name: None,
        start_date: None,
        end_date: None,
        description: None,
        followers: vec![],
        labels: vec![],
        group_ids: vec![],
    }
}

#[test]
fn wizard_minimal_fields() {
    let prompter = MockPrompter::new(vec![
        MockAnswer::Text("Sprint 24".into()),  // name
        MockAnswer::Text("2026-03-02".into()), // start_date
        MockAnswer::Text("2026-03-15".into()), // end_date
        MockAnswer::OptionalText(None),        // description
        MockAnswer::MultiSelect(vec![]),       // followers
        MockAnswer::List(vec![]),              // labels
        MockAnswer::List(vec![]),              // group_ids
        MockAnswer::Confirm(true),
    ]);

    let args = empty_interactive_args();
    let result = run_wizard(&args, &prompter, &test_members()).unwrap();

    assert_eq!(result.name.as_deref(), Some("Sprint 24"));
    assert_eq!(result.start_date.as_deref(), Some("2026-03-02"));
    assert_eq!(result.end_date.as_deref(), Some("2026-03-15"));
    assert_eq!(result.description, None);
    assert!(result.followers.is_empty());
    assert!(result.labels.is_empty());
    assert!(result.group_ids.is_empty());
    assert!(!result.interactive);
}

#[test]
fn wizard_all_fields() {
    let prompter = MockPrompter::new(vec![
        MockAnswer::Text("Sprint 25".into()),
        MockAnswer::Text("2026-03-16".into()),
        MockAnswer::Text("2026-03-29".into()),
        MockAnswer::OptionalText(Some("Full sprint".into())),
        MockAnswer::MultiSelect(vec!["@alice".into()]),
        MockAnswer::List(vec!["backend".into()]),
        MockAnswer::List(vec!["aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into()]),
        MockAnswer::Confirm(true),
    ]);

    let args = empty_interactive_args();
    let result = run_wizard(&args, &prompter, &test_members()).unwrap();

    assert_eq!(result.name.as_deref(), Some("Sprint 25"));
    assert_eq!(result.start_date.as_deref(), Some("2026-03-16"));
    assert_eq!(result.end_date.as_deref(), Some("2026-03-29"));
    assert_eq!(result.description.as_deref(), Some("Full sprint"));
    assert_eq!(result.followers, vec!["@alice"]);
    assert_eq!(result.labels, vec!["backend"]);
    assert_eq!(result.group_ids.len(), 1);
}

#[test]
fn wizard_abort() {
    let prompter = MockPrompter::new(vec![
        MockAnswer::Text("Aborted Sprint".into()),
        MockAnswer::Text("2026-04-01".into()),
        MockAnswer::Text("2026-04-14".into()),
        MockAnswer::OptionalText(None),
        MockAnswer::MultiSelect(vec![]),
        MockAnswer::List(vec![]),
        MockAnswer::List(vec![]),
        MockAnswer::Confirm(false),
    ]);

    let args = empty_interactive_args();
    let result = run_wizard(&args, &prompter, &test_members());
    let err = result.err().expect("expected error");
    assert_eq!(err.to_string(), "Aborted");
}

#[test]
fn wizard_pre_populated_fields_skip_prompts() {
    // Name, start_date, end_date are all pre-populated → skip those prompts
    let prompter = MockPrompter::new(vec![
        MockAnswer::OptionalText(None),  // description
        MockAnswer::MultiSelect(vec![]), // followers
        MockAnswer::List(vec![]),        // labels
        MockAnswer::List(vec![]),        // group_ids
        MockAnswer::Confirm(true),
    ]);

    let mut args = make_create_args("Sprint CLI", "2026-05-01", "2026-05-14");
    args.interactive = true;
    let result = run_wizard(&args, &prompter, &test_members()).unwrap();

    assert_eq!(result.name.as_deref(), Some("Sprint CLI"));
    assert_eq!(result.start_date.as_deref(), Some("2026-05-01"));
    assert_eq!(result.end_date.as_deref(), Some("2026-05-14"));
}
