use shortcut_cli::project;
use std::path::Path;
use std::process::{Command, Output};

fn config_dir(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_shortcut"))
        .arg("config-dir")
        .args(args)
        .env("HOME", home)
        .env_remove("SHORTCUT_API_TOKEN")
        .current_dir(cwd)
        .output()
        .unwrap()
}

fn stdout(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn prints_registered_directory_from_subdirectory_without_token() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let (root, _) = project::init_in(home.path(), project.path()).unwrap();
    let nested = project.path().join("nested directory");
    std::fs::create_dir(&nested).unwrap();

    let path = stdout(config_dir(home.path(), &nested, &[]));

    assert_eq!(path, format!("{}\n", root.config_dir().display()));
    assert!(!root.token_path().exists());
    assert!(root.config_dir().is_dir());
}

#[test]
fn reports_paths_in_json_and_toon_without_exposing_token() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let (root, _) = project::init_in(home.path(), project.path()).unwrap();
    std::fs::write(root.token_path(), "private-test-token").unwrap();
    let expected = serde_json::json!({
        "config_dir": root.config_dir(),
        "token_path": root.token_path(),
        "cache_dir": root.cache_dir(),
    });

    let json = stdout(config_dir(home.path(), project.path(), &["--json"]));
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&json).unwrap(),
        expected
    );
    let toon = stdout(config_dir(home.path(), project.path(), &["--toon"]));
    assert_eq!(toon, format!("{}\n", toon::encode(&expected, None)));
    assert!(!json.contains("private-test-token"));
    assert!(!toon.contains("private-test-token"));
    assert_eq!(
        std::fs::read_to_string(root.token_path()).unwrap(),
        "private-test-token"
    );
}

#[test]
fn supports_format_quiet_and_dry_run() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let (root, _) = project::init_in(home.path(), project.path()).unwrap();

    let formatted = stdout(config_dir(
        home.path(),
        project.path(),
        &["--format", "{token_path}"],
    ));
    assert_eq!(formatted, format!("{}\n", root.token_path().display()));
    for flag in ["--quiet", "--dry-run"] {
        assert_eq!(
            stdout(config_dir(home.path(), project.path(), &[flag])),
            format!("{}\n", root.config_dir().display())
        );
    }
}

#[test]
fn unregistered_project_fails_without_creating_config() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let output = config_dir(home.path(), project.path(), &[]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("No project registered"));
    assert!(!home.path().join(".shortcut").exists());
}

#[test]
fn help_explains_recovery_and_discovery() {
    let output = Command::new(env!("CARGO_BIN_EXE_shortcut"))
        .args(["config-dir", "--help"])
        .env_remove("HOME")
        .output()
        .unwrap();
    let help = stdout(output);
    assert!(help.contains("linked Git worktree"));
    assert!(help.contains("token_path"));
    assert!(help.contains("No token or network access is required"));
}
