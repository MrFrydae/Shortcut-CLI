mod support;

#[path = "template/create.rs"]
mod create;
#[path = "template/delete.rs"]
mod delete;
#[path = "template/get.rs"]
mod get;
#[path = "template/list.rs"]
mod list;
#[path = "template/update.rs"]
mod update;
#[path = "template/use_template.rs"]
mod use_template;

pub const TEMPLATE_UUID: &str = "aaaaaaaa-1111-2222-3333-444444444444";
pub const TEMPLATE_UUID2: &str = "bbbbbbbb-1111-2222-3333-444444444444";
