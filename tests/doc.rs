mod support;

#[path = "doc/create.rs"]
mod create;
#[path = "doc/delete.rs"]
mod delete;
#[path = "doc/epics.rs"]
mod epics;
#[path = "doc/get.rs"]
mod get;
#[path = "doc/link.rs"]
mod link;
#[path = "doc/list.rs"]
mod list;
#[path = "doc/unlink.rs"]
mod unlink;
#[path = "doc/update.rs"]
mod update;

pub const DOC_UUID: &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
pub const DOC_UUID2: &str = "11111111-2222-3333-4444-555555555555";
