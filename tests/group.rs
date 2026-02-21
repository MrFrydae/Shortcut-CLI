mod support;

#[path = "group/create.rs"]
mod create;
#[path = "group/get.rs"]
mod get;
#[path = "group/list.rs"]
mod list;
#[path = "group/stories.rs"]
mod stories;
#[path = "group/update.rs"]
mod update;

pub const UUID_GROUP1: &str = "11111111-1111-1111-1111-111111111111";
pub const UUID_GROUP2: &str = "22222222-2222-2222-2222-222222222222";
pub const UUID_ALICE: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
