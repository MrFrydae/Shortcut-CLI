#[allow(clippy::all, unused)]
mod inner {
    include!(concat!(env!("OUT_DIR"), "/shortcut_api.rs"));
}

// Re-exported types will be used as subcommands are implemented.
#[allow(unused_imports)]
pub use inner::*;
