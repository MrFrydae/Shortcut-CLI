use crate::support::make_output;
use sc::commands::template;

#[tokio::test]
async fn init_stdout_prints_content() {
    use sc::output::{ColorMode, OutputConfig, OutputMode};

    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let args = template::init::InitArgs {
        path: None,
        stdout: true,
    };
    let result = template::init::run(&args, &out).await;
    assert!(result.is_ok());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(output.contains("<!-- BEGIN-STL-AGENT-INSTRUCTIONS -->"));
    assert!(output.contains("<!-- END-STL-AGENT-INSTRUCTIONS -->"));
    assert!(output.contains("ACTION×ENTITY MATRIX"));
    assert!(output.contains("$var(name)"));
    assert!(output.contains("$ref(alias)"));
}

#[tokio::test]
async fn init_creates_claude_md() {
    let out = make_output();
    let tmp = tempfile::tempdir().unwrap();

    let args = template::init::InitArgs {
        path: Some(tmp.path().to_path_buf()),
        stdout: false,
    };
    let result = template::init::run(&args, &out).await;
    assert!(result.is_ok());

    let claude_md = tmp.path().join("CLAUDE.md");
    assert!(claude_md.exists());
    let content = std::fs::read_to_string(&claude_md).unwrap();
    assert!(content.contains("<!-- BEGIN-STL-AGENT-INSTRUCTIONS -->"));
    assert!(content.contains("<!-- END-STL-AGENT-INSTRUCTIONS -->"));
}

#[tokio::test]
async fn init_appends_to_existing_claude_md() {
    let out = make_output();
    let tmp = tempfile::tempdir().unwrap();

    let claude_md = tmp.path().join("CLAUDE.md");
    std::fs::write(&claude_md, "# My Project\n\nExisting content.\n").unwrap();

    let args = template::init::InitArgs {
        path: Some(tmp.path().to_path_buf()),
        stdout: false,
    };
    let result = template::init::run(&args, &out).await;
    assert!(result.is_ok());

    let content = std::fs::read_to_string(&claude_md).unwrap();
    assert!(content.starts_with("# My Project"));
    assert!(content.contains("Existing content."));
    assert!(content.contains("<!-- BEGIN-STL-AGENT-INSTRUCTIONS -->"));
}

#[tokio::test]
async fn init_idempotent_no_duplicate() {
    use sc::output::{ColorMode, OutputConfig, OutputMode};

    let tmp = tempfile::tempdir().unwrap();

    // First run
    let out = make_output();
    let args = template::init::InitArgs {
        path: Some(tmp.path().to_path_buf()),
        stdout: false,
    };
    template::init::run(&args, &out).await.unwrap();

    let content_after_first = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();

    // Second run
    let (out2, buf2) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let args2 = template::init::InitArgs {
        path: Some(tmp.path().to_path_buf()),
        stdout: false,
    };
    let result = template::init::run(&args2, &out2).await;
    assert!(result.is_ok());

    let content_after_second = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert_eq!(content_after_first, content_after_second);

    let output = String::from_utf8(buf2.lock().unwrap().clone()).unwrap();
    assert!(output.contains("already present"));
}
