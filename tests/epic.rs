mod support;

#[path = "epic/comment.rs"]
mod comment;
#[path = "epic/create.rs"]
mod create;
#[path = "epic/delete.rs"]
mod delete;
#[path = "epic/docs.rs"]
mod docs;
#[path = "epic/get.rs"]
mod get;
#[path = "epic/list.rs"]
mod list;
#[path = "epic/update.rs"]
mod update;
#[path = "epic/wizard.rs"]
mod wizard;

pub const UUID_ALICE: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

pub fn make_list_args(desc: bool) -> sc::commands::epic::EpicArgs {
    sc::commands::epic::EpicArgs {
        action: sc::commands::epic::EpicAction::List { desc },
    }
}

pub fn make_update_args(id: i64) -> sc::commands::epic::UpdateArgs {
    sc::commands::epic::UpdateArgs {
        id,
        name: None,
        description: None,
        deadline: None,
        archived: None,
        epic_state_id: None,
        labels: vec![],
        objective_ids: vec![],
        owner_ids: vec![],
        follower_ids: vec![],
        requested_by_id: None,
    }
}

pub fn make_create_args(name: &str) -> sc::commands::epic::CreateArgs {
    sc::commands::epic::CreateArgs {
        interactive: false,
        name: Some(name.to_string()),
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
