use std::cell::RefCell;
use std::error::Error;

use sc::commands::story::{commit, git};
use sc::output::{ColorMode, OutputConfig, OutputMode};

struct TestGitRunner {
    branch: String,
    commit_output: String,
    committed_args: RefCell<Option<Vec<String>>>,
}

impl TestGitRunner {
    fn new(branch: &str, commit_output: &str) -> Self {
        Self {
            branch: branch.to_string(),
            commit_output: commit_output.to_string(),
            committed_args: RefCell::new(None),
        }
    }
}

impl git::GitRunner for TestGitRunner {
    fn current_branch(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.branch.clone())
    }

    fn checkout_new_branch(&self, _branch: &str) -> Result<(), Box<dyn Error>> {
        unimplemented!("not used in commit tests")
    }

    fn commit(&self, args: &[&str]) -> Result<String, Box<dyn Error>> {
        *self.committed_args.borrow_mut() = Some(args.iter().map(|s| s.to_string()).collect());
        Ok(self.commit_output.clone())
    }
}

#[test]
fn explicit_id_prepends_prefix() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let git_runner = TestGitRunner::new("main", "[main abc1234] [sc-123] Fix bug\n 1 file changed");

    let args = commit::CommitArgs {
        id: Some(123),
        message: "Fix bug".to_string(),
        extra_args: vec![],
    };

    let result = commit::run_with_git(&args, &out, &git_runner);
    assert!(result.is_ok());

    let committed = git_runner.committed_args.borrow();
    let committed = committed.as_ref().unwrap();
    assert_eq!(committed[0], "-m");
    assert_eq!(committed[1], "[sc-123] Fix bug");

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(!output.is_empty());
}

#[test]
fn extract_id_from_branch() {
    let (out, _buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let git_runner = TestGitRunner::new("feature/sc-456-login-fix", "");

    let args = commit::CommitArgs {
        id: None,
        message: "Update tests".to_string(),
        extra_args: vec![],
    };

    let result = commit::run_with_git(&args, &out, &git_runner);
    assert!(result.is_ok());

    let committed = git_runner.committed_args.borrow();
    let committed = committed.as_ref().unwrap();
    assert_eq!(committed[1], "[sc-456] Update tests");
}

#[test]
fn unrecognized_branch_returns_error() {
    let (out, _buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let git_runner = TestGitRunner::new("main", "");

    let args = commit::CommitArgs {
        id: None,
        message: "Fix bug".to_string(),
        extra_args: vec![],
    };

    let result = commit::run_with_git(&args, &out, &git_runner);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Could not detect story ID"));
    assert!(err.contains("main"));
}

#[test]
fn extra_args_passed_through() {
    let (out, _buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let git_runner = TestGitRunner::new("main", "");

    let args = commit::CommitArgs {
        id: Some(789),
        message: "WIP".to_string(),
        extra_args: vec!["--no-verify".to_string(), "--amend".to_string()],
    };

    let result = commit::run_with_git(&args, &out, &git_runner);
    assert!(result.is_ok());

    let committed = git_runner.committed_args.borrow();
    let committed = committed.as_ref().unwrap();
    assert_eq!(committed.len(), 4);
    assert_eq!(committed[0], "-m");
    assert_eq!(committed[1], "[sc-789] WIP");
    assert_eq!(committed[2], "--no-verify");
    assert_eq!(committed[3], "--amend");
}
