use std::process::Command;

fn main() {
    // Point git to our version-controlled hooks directory.
    // This mirrors npm prepare / husky — anyone who clones and builds
    // gets hooks set up automatically.
    let _ = Command::new("git")
        .args(["config", "core.hooksPath", ".githooks"])
        .status();

    // Only re-run this script when build.rs itself changes.
    println!("cargo:rerun-if-changed=build.rs");
}
