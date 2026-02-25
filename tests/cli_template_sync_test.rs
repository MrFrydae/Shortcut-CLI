use clap::Parser;
use shortcut_cli::cli::{Cli, Command};
use shortcut_cli::commands::template::TemplateAction;

#[test]
fn parses_template_sync_command_and_flags() {
    let cli = Cli::parse_from([
        "shortcut",
        "--dry-run",
        "template",
        "sync",
        "plan.shortcut.yml",
        "--state",
        "plan.state.json",
        "--prune",
        "--confirm",
        "--var",
        "sprint=Sprint 24",
        "--var",
        "owner=Team A",
    ]);

    assert!(cli.dry_run);
    match cli.command {
        Command::Template(args) => match args.action {
            TemplateAction::Sync(sync_args) => {
                assert_eq!(sync_args.file, "plan.shortcut.yml");
                assert_eq!(sync_args.state.as_deref(), Some("plan.state.json"));
                assert!(sync_args.prune);
                assert!(sync_args.confirm);
                assert_eq!(
                    sync_args.vars,
                    vec![
                        ("sprint".to_string(), "Sprint 24".to_string()),
                        ("owner".to_string(), "Team A".to_string())
                    ]
                );
            }
            _ => panic!("expected template sync action"),
        },
        _ => panic!("expected template command"),
    }
}
