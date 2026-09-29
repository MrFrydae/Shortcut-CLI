use shortcut_cli::project;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-c")
        .arg("user.name=Test")
        .arg("-c")
        .arg("user.email=test@example.com")
        .arg("-c")
        .arg("commit.gpgsign=false")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn make_worktree(dir: &Path) -> (PathBuf, PathBuf) {
    let main = dir.join("main project");
    let worktree = dir.join("linked worktree");
    std::fs::create_dir(&main).unwrap();
    git(&main, &["init", "-q"]);
    git(&main, &["commit", "--allow-empty", "-m", "initial"]);
    git(
        &main,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "linked",
            worktree.to_str().unwrap(),
        ],
    );
    (main, worktree)
}

#[test]
fn init_creates_structure() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let (root, _) = project::init_in(home.path(), project.path()).unwrap();

    assert!(root.token_path().parent().unwrap().is_dir());
    assert!(root.cache_dir().is_dir());
}

#[test]
fn init_errors_if_already_exists() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    project::init_in(home.path(), project.path()).unwrap();

    let result = project::init_in(home.path(), project.path());
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("already initialized"), "got: {err}");
}

#[test]
fn discover_finds_existing() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    project::init_in(home.path(), project.path()).unwrap();

    let root = project::discover_in(home.path(), project.path()).unwrap();
    assert!(root.cache_dir().is_dir());
}

#[test]
fn discover_returns_not_found() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let result = project::discover_in(home.path(), project.path());
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("No project registered for this directory"),
        "got: {err}"
    );
}

#[test]
fn path_accessors() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let (root, _) = project::init_in(home.path(), project.path()).unwrap();

    assert!(root.token_path().ends_with("token"));
    assert!(root.cache_dir().ends_with("cache"));
}

#[test]
fn same_project_gets_same_subdir() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let (root1, _) = project::init_in(home.path(), project.path()).unwrap();
    let root2 = project::discover_in(home.path(), project.path()).unwrap();

    assert_eq!(root1.cache_dir(), root2.cache_dir());
}

#[test]
fn discover_finds_from_subdirectory() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let (init_root, _) = project::init_in(home.path(), project.path()).unwrap();

    let subdir = project.path().join("a").join("b").join("c");
    std::fs::create_dir_all(&subdir).unwrap();

    let discovered = project::discover_in(home.path(), &subdir).unwrap();
    assert_eq!(init_root.cache_dir(), discovered.cache_dir());
}

#[test]
fn discover_or_init_creates_when_missing() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();

    let root = project::discover_or_init_in(home.path(), project.path()).unwrap();
    assert!(root.cache_dir().is_dir());
}

#[test]
fn discover_or_init_finds_existing() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let (init_root, _) = project::init_in(home.path(), project.path()).unwrap();

    let root = project::discover_or_init_in(home.path(), project.path()).unwrap();
    assert_eq!(init_root.cache_dir(), root.cache_dir());
}

#[test]
fn different_projects_get_different_subdirs() {
    let home = tempfile::tempdir().unwrap();
    let project_a = tempfile::tempdir().unwrap();
    let project_b = tempfile::tempdir().unwrap();

    let (root_a, _) = project::init_in(home.path(), project_a.path()).unwrap();
    let (root_b, _) = project::init_in(home.path(), project_b.path()).unwrap();

    assert_ne!(root_a.cache_dir(), root_b.cache_dir());
}

#[test]
fn linked_worktree_finds_main_project_token_from_subdirectory() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let (main, worktree) = make_worktree(dir.path());
    project::init_in(&home, dir.path()).unwrap();
    let (main_root, _) = project::init_in(&home, &main).unwrap();
    std::fs::write(main_root.token_path(), "main-token").unwrap();

    let nested = worktree.join("a").join("b");
    std::fs::create_dir_all(&nested).unwrap();
    let discovered = project::discover_in(&home, &nested).unwrap();

    assert_eq!(discovered.token_path(), main_root.token_path());
    assert_eq!(
        std::fs::read_to_string(discovered.token_path()).unwrap(),
        "main-token"
    );
}

#[test]
fn login_from_unregistered_worktree_initializes_main_project() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let (main, worktree) = make_worktree(dir.path());

    let nested = worktree.join("nested");
    std::fs::create_dir(&nested).unwrap();
    let root = project::discover_or_init_in(&home, &nested).unwrap();
    let main_root = project::discover_in(&home, &main).unwrap();
    assert_eq!(root.token_path(), main_root.token_path());
}

#[test]
fn linked_worktree_registration_takes_precedence() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let main = dir.path().join("main");
    let worktree = dir.path().join("linked");
    std::fs::create_dir(&main).unwrap();
    std::fs::create_dir(&worktree).unwrap();
    let (linked_root, _) = project::init_in(&home, &worktree).unwrap();
    git(&main, &["init", "-q"]);
    git(&main, &["commit", "--allow-empty", "-m", "initial"]);
    git(
        &main,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "linked",
            worktree.to_str().unwrap(),
        ],
    );
    let (main_root, _) = project::init_in(&home, &main).unwrap();

    let discovered = project::discover_in(&home, &worktree).unwrap();
    assert_ne!(linked_root.token_path(), main_root.token_path());
    assert_eq!(discovered.token_path(), linked_root.token_path());
}
