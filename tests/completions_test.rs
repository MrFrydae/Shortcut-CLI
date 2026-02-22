use clap_complete::Shell;
use sc::commands::completions;

fn generate(shell: Shell) -> String {
    let mut buf = Vec::new();
    completions::run(shell, &mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

#[test]
fn generates_bash_completions() {
    let output = generate(Shell::Bash);
    assert!(
        output.contains("_sc"),
        "bash completions should contain _sc function"
    );
}

#[test]
fn generates_zsh_completions() {
    let output = generate(Shell::Zsh);
    assert!(
        output.contains("#compdef sc"),
        "zsh completions should contain #compdef sc"
    );
}

#[test]
fn generates_fish_completions() {
    let output = generate(Shell::Fish);
    assert!(
        output.contains("complete -c sc"),
        "fish completions should contain 'complete -c sc'"
    );
}

#[test]
fn generates_powershell_completions() {
    let output = generate(Shell::PowerShell);
    assert!(
        !output.is_empty(),
        "powershell completions should not be empty"
    );
    assert!(
        output.contains("sc"),
        "powershell completions should reference 'sc'"
    );
}

#[test]
fn completions_include_all_subcommands() {
    let output = generate(Shell::Bash);
    let subcommands = [
        "init",
        "login",
        "category",
        "custom-field",
        "doc",
        "epic",
        "group",
        "iteration",
        "label",
        "member",
        "objective",
        "project",
        "search",
        "story",
        "template",
        "workflow",
    ];
    for cmd in &subcommands {
        assert!(
            output.contains(cmd),
            "bash completions should contain subcommand '{cmd}'"
        );
    }
}

#[test]
fn completions_do_not_include_hidden_completions_command() {
    let output = generate(Shell::Bash);
    assert!(
        !output.contains("completions"),
        "bash completions should not expose the hidden 'completions' command"
    );
}
