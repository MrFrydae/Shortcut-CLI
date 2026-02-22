mod support;

#[path = "story/branch.rs"]
mod branch;
#[path = "story/comment.rs"]
mod comment;
#[path = "story/commit.rs"]
mod commit;
#[path = "story/create.rs"]
mod create;
#[path = "story/custom_field.rs"]
mod custom_field;
#[path = "story/delete.rs"]
mod delete;
#[path = "story/get.rs"]
mod get;
#[path = "story/history.rs"]
mod history;
#[path = "story/link.rs"]
mod link;
#[path = "story/list.rs"]
mod list;
#[path = "story/task.rs"]
mod task;
#[path = "story/update.rs"]
mod update;
#[path = "story/wizard.rs"]
mod wizard;

pub const UUID_ALICE: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
pub const UUID_BOB: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";

pub const UUID_FIELD_1: &str = "11111111-1111-1111-1111-111111111111";
pub const UUID_VAL_A: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaab";
pub const UUID_VAL_B: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb2";

pub fn make_create_args(name: &str) -> sc::commands::story::CreateArgs {
    sc::commands::story::CreateArgs {
        interactive: false,
        name: Some(name.to_string()),
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

pub fn make_update_args(id: i64) -> sc::commands::story::UpdateArgs {
    sc::commands::story::UpdateArgs {
        id,
        name: None,
        description: None,
        story_type: None,
        owner: vec![],
        state: None,
        epic_id: None,
        estimate: None,
        labels: vec![],
        iteration_id: None,
        custom_fields: vec![],
    }
}

pub fn make_list_args() -> sc::commands::story::ListArgs {
    sc::commands::story::ListArgs {
        owner: None,
        state: None,
        epic_id: None,
        story_type: None,
        label: None,
        project_id: None,
        limit: 25,
        desc: false,
    }
}
