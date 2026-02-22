use sc::commands::epic::wizard::run_wizard;
use sc::interactive::{IdChoice, MemberChoice, MockAnswer, MockPrompter, UuidChoice};

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

fn test_group_choices() -> Vec<UuidChoice> {
    vec![
        UuidChoice {
            display: "Backend Team (@backend)".into(),
            id: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        },
        UuidChoice {
            display: "Frontend Team (@frontend)".into(),
            id: uuid::Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
        },
    ]
}

fn test_objective_choices() -> Vec<IdChoice> {
    vec![
        IdChoice {
            display: "North Star [in progress] (#1)".into(),
            id: 1,
        },
        IdChoice {
            display: "Q2 Goals [to do] (#2)".into(),
            id: 2,
        },
    ]
}

fn test_epic_states() -> Vec<String> {
    vec!["To Do".into(), "In Progress".into(), "Done".into()]
}

/// Helper to build a default CreateArgs with interactive=true and no name.
fn empty_interactive_args() -> sc::commands::epic::CreateArgs {
    sc::commands::epic::CreateArgs {
        interactive: true,
        name: None,
        description: None,
        state: None,
        deadline: None,
        owners: vec![],
        group_ids: vec![],
        labels: vec![],
        objective_ids: vec![],
        followers: vec![],
        requested_by: None,
    }
}

#[test]
fn wizard_minimal_fields() {
    let prompter = MockPrompter::new(vec![
        MockAnswer::Text("My Epic".into()), // name
        MockAnswer::OptionalText(None),     // description
        MockAnswer::OptionalSelect(None),   // state
        MockAnswer::OptionalText(None),     // deadline
        MockAnswer::MultiSelect(vec![]),    // owners
        MockAnswer::List(vec![]),           // labels
        MockAnswer::OptionalUuid(None),     // group_ids
        MockAnswer::MultiSelectId(vec![]),  // objective_ids
        MockAnswer::MultiSelect(vec![]),    // followers
        MockAnswer::OptionalText(None),     // requested_by
        MockAnswer::Confirm(true),
    ]);

    let args = empty_interactive_args();
    let result = run_wizard(
        &args,
        &prompter,
        &test_members(),
        &test_epic_states(),
        &test_group_choices(),
        &test_objective_choices(),
    )
    .unwrap();

    assert_eq!(result.name.as_deref(), Some("My Epic"));
    assert_eq!(result.description, None);
    assert_eq!(result.state, None);
    assert_eq!(result.deadline, None);
    assert!(result.owners.is_empty());
    assert!(result.labels.is_empty());
    assert!(result.objective_ids.is_empty());
    assert!(result.followers.is_empty());
    assert_eq!(result.requested_by, None);
    assert!(!result.interactive);
}

#[test]
fn wizard_all_fields() {
    let prompter = MockPrompter::new(vec![
        MockAnswer::Text("Launch V2".into()),                   // name
        MockAnswer::OptionalText(Some("Major release".into())), // description
        MockAnswer::OptionalSelect(Some("In Progress".into())), // state
        MockAnswer::OptionalText(Some("2025-12-31T00:00:00Z".into())), // deadline
        MockAnswer::MultiSelect(vec!["@alice".into()]),         // owners
        MockAnswer::List(vec!["frontend".into()]),              // labels
        MockAnswer::OptionalUuid(Some(
            uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        )), // group_ids
        MockAnswer::MultiSelectId(vec![1, 2]),                  // objective_ids
        MockAnswer::MultiSelect(vec!["@bob".into()]),           // followers
        MockAnswer::OptionalText(Some("@carol".into())),        // requested_by
        MockAnswer::Confirm(true),
    ]);

    let args = empty_interactive_args();
    let result = run_wizard(
        &args,
        &prompter,
        &test_members(),
        &test_epic_states(),
        &test_group_choices(),
        &test_objective_choices(),
    )
    .unwrap();

    assert_eq!(result.name.as_deref(), Some("Launch V2"));
    assert_eq!(result.description.as_deref(), Some("Major release"));
    assert_eq!(result.state.as_deref(), Some("In Progress"));
    assert_eq!(result.deadline.as_deref(), Some("2025-12-31T00:00:00Z"));
    assert_eq!(result.owners, vec!["@alice"]);
    assert_eq!(result.labels, vec!["frontend"]);
    assert_eq!(
        result.group_ids,
        vec!["aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"]
    );
    assert_eq!(result.objective_ids, vec![1, 2]);
    assert_eq!(result.followers, vec!["@bob"]);
    assert_eq!(result.requested_by.as_deref(), Some("@carol"));
}

#[test]
fn wizard_abort() {
    let prompter = MockPrompter::new(vec![
        MockAnswer::Text("Aborted Epic".into()),
        MockAnswer::OptionalText(None),
        MockAnswer::OptionalSelect(None),
        MockAnswer::OptionalText(None),
        MockAnswer::MultiSelect(vec![]),
        MockAnswer::List(vec![]),
        MockAnswer::OptionalUuid(None),
        MockAnswer::MultiSelectId(vec![]),
        MockAnswer::MultiSelect(vec![]),
        MockAnswer::OptionalText(None),
        MockAnswer::Confirm(false),
    ]);

    let args = empty_interactive_args();
    let result = run_wizard(
        &args,
        &prompter,
        &test_members(),
        &test_epic_states(),
        &test_group_choices(),
        &test_objective_choices(),
    );
    let err = result.err().expect("expected error");
    assert_eq!(err.to_string(), "Aborted");
}

#[test]
fn wizard_pre_populated_fields_skip_prompts() {
    let prompter = MockPrompter::new(vec![
        MockAnswer::OptionalText(None),    // description
        MockAnswer::OptionalSelect(None),  // state
        MockAnswer::OptionalText(None),    // deadline
        MockAnswer::List(vec![]),          // labels
        MockAnswer::OptionalUuid(None),    // group_ids
        MockAnswer::MultiSelectId(vec![]), // objective_ids
        MockAnswer::MultiSelect(vec![]),   // followers
        MockAnswer::OptionalText(None),    // requested_by
        MockAnswer::Confirm(true),
    ]);

    let mut args = make_create_args("CLI Epic");
    args.interactive = true;
    args.owners = vec!["@alice".into()];
    let result = run_wizard(
        &args,
        &prompter,
        &test_members(),
        &test_epic_states(),
        &test_group_choices(),
        &test_objective_choices(),
    )
    .unwrap();

    assert_eq!(result.name.as_deref(), Some("CLI Epic"));
    assert_eq!(result.owners, vec!["@alice"]);
}
