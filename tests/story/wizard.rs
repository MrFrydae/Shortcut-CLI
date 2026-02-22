use sc::commands::story::wizard::{WizardChoices, run_wizard};
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

fn test_workflow_states() -> Vec<String> {
    vec!["Unstarted".into(), "In Progress".into(), "Done".into()]
}

fn test_story_types() -> Vec<&'static str> {
    vec!["feature", "bug", "chore"]
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

fn test_epic_choices() -> Vec<IdChoice> {
    vec![
        IdChoice {
            display: "Backend Epic (#42)".into(),
            id: 42,
        },
        IdChoice {
            display: "Frontend Epic (#99)".into(),
            id: 99,
        },
    ]
}

fn test_iteration_choices() -> Vec<IdChoice> {
    vec![
        IdChoice {
            display: "Sprint 1 [started] (#10)".into(),
            id: 10,
        },
        IdChoice {
            display: "Sprint 2 [unstarted] (#20)".into(),
            id: 20,
        },
    ]
}

/// Helper to build a default CreateArgs with interactive=true and no name.
fn empty_interactive_args() -> sc::commands::story::CreateArgs {
    sc::commands::story::CreateArgs {
        interactive: true,
        name: None,
        description: None,
        story_type: None,
        owner: vec![],
        state: None,
        epic_id: None,
        group_id: None,
        estimate: None,
        labels: vec![],
        iteration_id: None,
        custom_fields: vec![],
    }
}

#[test]
fn wizard_minimal_fields() {
    let prompter = MockPrompter::new(vec![
        MockAnswer::Text("My Story".into()), // name
        MockAnswer::OptionalText(None),      // description
        MockAnswer::OptionalSelect(None),    // story_type
        MockAnswer::MultiSelect(vec![]),     // owners
        MockAnswer::OptionalSelect(None),    // state
        MockAnswer::OptionalI64(None),       // epic_id
        MockAnswer::OptionalUuid(None),      // group_id
        MockAnswer::OptionalI64(None),       // estimate
        MockAnswer::List(vec![]),            // labels
        MockAnswer::OptionalI64(None),       // iteration_id
        MockAnswer::List(vec![]),            // custom_fields
        MockAnswer::Confirm(true),           // confirm
    ]);

    let args = empty_interactive_args();
    let members = test_members();
    let workflow_states = test_workflow_states();
    let story_types = test_story_types();
    let epic_choices = test_epic_choices();
    let iteration_choices = test_iteration_choices();
    let group_choices = test_group_choices();
    let choices = WizardChoices {
        members: &members,
        workflow_states: &workflow_states,
        story_types: &story_types,
        epic_choices: &epic_choices,
        iteration_choices: &iteration_choices,
        group_choices: &group_choices,
    };
    let result = run_wizard(&args, &prompter, &choices).unwrap();

    assert_eq!(result.name.as_deref(), Some("My Story"));
    assert_eq!(result.description, None);
    assert_eq!(result.story_type, None);
    assert!(result.owner.is_empty());
    assert_eq!(result.state, None);
    assert_eq!(result.epic_id, None);
    assert_eq!(result.estimate, None);
    assert!(result.labels.is_empty());
    assert_eq!(result.iteration_id, None);
    assert!(result.custom_fields.is_empty());
    assert!(!result.interactive);
}

#[test]
fn wizard_all_fields() {
    let prompter = MockPrompter::new(vec![
        MockAnswer::Text("Fix login bug".into()), // name
        MockAnswer::OptionalText(Some("Crashes on Safari".into())), // description
        MockAnswer::OptionalSelect(Some("bug".into())), // story_type
        MockAnswer::MultiSelect(vec!["@alice".into(), "@bob".into()]), // owners
        MockAnswer::OptionalSelect(Some("In Progress".into())), // state
        MockAnswer::OptionalI64(Some(42)),        // epic_id
        MockAnswer::OptionalUuid(Some(
            uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        )), // group_id
        MockAnswer::OptionalI64(Some(5)),         // estimate
        MockAnswer::List(vec!["backend".into(), "urgent".into()]), // labels
        MockAnswer::OptionalI64(Some(10)),        // iteration_id
        MockAnswer::List(vec!["Priority=High".into()]), // custom_fields
        MockAnswer::Confirm(true),                // confirm
    ]);

    let args = empty_interactive_args();
    let members = test_members();
    let workflow_states = test_workflow_states();
    let story_types = test_story_types();
    let epic_choices = test_epic_choices();
    let iteration_choices = test_iteration_choices();
    let group_choices = test_group_choices();
    let choices = WizardChoices {
        members: &members,
        workflow_states: &workflow_states,
        story_types: &story_types,
        epic_choices: &epic_choices,
        iteration_choices: &iteration_choices,
        group_choices: &group_choices,
    };
    let result = run_wizard(&args, &prompter, &choices).unwrap();

    assert_eq!(result.name.as_deref(), Some("Fix login bug"));
    assert_eq!(result.description.as_deref(), Some("Crashes on Safari"));
    assert_eq!(result.story_type.as_deref(), Some("bug"));
    assert_eq!(result.owner, vec!["@alice", "@bob"]);
    assert_eq!(result.state.as_deref(), Some("In Progress"));
    assert_eq!(result.epic_id, Some(42));
    assert_eq!(
        result.group_id.as_deref(),
        Some("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
    );
    assert_eq!(result.estimate, Some(5));
    assert_eq!(result.labels, vec!["backend", "urgent"]);
    assert_eq!(result.iteration_id, Some(10));
    assert_eq!(result.custom_fields, vec!["Priority=High"]);
}

#[test]
fn wizard_abort() {
    let prompter = MockPrompter::new(vec![
        MockAnswer::Text("Aborted Story".into()),
        MockAnswer::OptionalText(None),
        MockAnswer::OptionalSelect(None),
        MockAnswer::MultiSelect(vec![]),
        MockAnswer::OptionalSelect(None),
        MockAnswer::OptionalI64(None),
        MockAnswer::OptionalUuid(None),
        MockAnswer::OptionalI64(None),
        MockAnswer::List(vec![]),
        MockAnswer::OptionalI64(None),
        MockAnswer::List(vec![]),
        MockAnswer::Confirm(false), // abort
    ]);

    let args = empty_interactive_args();
    let members = test_members();
    let workflow_states = test_workflow_states();
    let story_types = test_story_types();
    let epic_choices = test_epic_choices();
    let iteration_choices = test_iteration_choices();
    let group_choices = test_group_choices();
    let choices = WizardChoices {
        members: &members,
        workflow_states: &workflow_states,
        story_types: &story_types,
        epic_choices: &epic_choices,
        iteration_choices: &iteration_choices,
        group_choices: &group_choices,
    };
    let result = run_wizard(&args, &prompter, &choices);
    let err = result.err().expect("expected error");
    assert_eq!(err.to_string(), "Aborted");
}

#[test]
fn wizard_pre_populated_fields_skip_prompts() {
    // Only need prompts for fields NOT pre-populated, plus confirm
    let prompter = MockPrompter::new(vec![
        MockAnswer::OptionalText(None),   // description (not pre-populated)
        MockAnswer::MultiSelect(vec![]),  // owners (not pre-populated)
        MockAnswer::OptionalSelect(None), // state (not pre-populated)
        MockAnswer::OptionalI64(None),    // epic_id (not pre-populated)
        MockAnswer::OptionalUuid(None),   // group_id (not pre-populated)
        MockAnswer::OptionalI64(None),    // estimate (not pre-populated)
        MockAnswer::List(vec![]),         // labels (not pre-populated)
        MockAnswer::OptionalI64(None),    // iteration_id (not pre-populated)
        MockAnswer::List(vec![]),         // custom_fields (not pre-populated)
        MockAnswer::Confirm(true),
    ]);

    let mut args = make_create_args("CLI Story");
    args.interactive = true;
    args.story_type = Some("chore".into());
    let members = test_members();
    let workflow_states = test_workflow_states();
    let story_types = test_story_types();
    let epic_choices = test_epic_choices();
    let iteration_choices = test_iteration_choices();
    let group_choices = test_group_choices();
    let choices = WizardChoices {
        members: &members,
        workflow_states: &workflow_states,
        story_types: &story_types,
        epic_choices: &epic_choices,
        iteration_choices: &iteration_choices,
        group_choices: &group_choices,
    };
    let result = run_wizard(&args, &prompter, &choices).unwrap();

    // Pre-populated values should be preserved
    assert_eq!(result.name.as_deref(), Some("CLI Story"));
    assert_eq!(result.story_type.as_deref(), Some("chore"));
}
