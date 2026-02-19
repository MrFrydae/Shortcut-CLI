use sc::project;

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
