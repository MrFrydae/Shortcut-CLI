mod support;

#[path = "search/all.rs"]
mod all;
#[path = "search/documents.rs"]
mod documents;
#[path = "search/epics.rs"]
mod epics;
#[path = "search/iterations.rs"]
mod iterations;
#[path = "search/milestones.rs"]
mod milestones;
#[path = "search/objectives.rs"]
mod objectives;
#[path = "search/stories.rs"]
mod stories;

pub fn make_query(query: &str) -> sc::commands::search::SearchQueryArgs {
    sc::commands::search::SearchQueryArgs {
        query: query.to_string(),
        page_size: 25,
        next: None,
        desc: false,
    }
}
