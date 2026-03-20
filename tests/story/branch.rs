use std::cell::RefCell;
use std::error::Error;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use shortcut_cli::api;
use shortcut_cli::commands::story::{branch, git};
use shortcut_cli::output::{ColorMode, OutputConfig, OutputMode};

use crate::support::full_story_json;

struct TestGitRunner {
    branch_exists: bool,
    checkout_result: Result<(), String>,
    checkout_new_branch_called: RefCell<bool>,
    checkout_branch_called: RefCell<bool>,
    checked_out_branch: RefCell<Option<String>>,
}

impl TestGitRunner {
    fn new(branch_exists: bool) -> Self {
        Self {
            branch_exists,
            checkout_result: Ok(()),
            checkout_new_branch_called: RefCell::new(false),
            checkout_branch_called: RefCell::new(false),
            checked_out_branch: RefCell::new(None),
        }
    }

    fn failing(branch_exists: bool, msg: &str) -> Self {
        Self {
            branch_exists,
            checkout_result: Err(msg.to_string()),
            checkout_new_branch_called: RefCell::new(false),
            checkout_branch_called: RefCell::new(false),
            checked_out_branch: RefCell::new(None),
        }
    }
}

impl git::GitRunner for TestGitRunner {
    fn current_branch(&self) -> Result<String, Box<dyn Error>> {
        unimplemented!("not used in branch tests")
    }

    fn branch_exists(&self, _branch: &str) -> Result<bool, Box<dyn Error>> {
        Ok(self.branch_exists)
    }

    fn checkout_branch(&self, branch: &str) -> Result<(), Box<dyn Error>> {
        *self.checkout_branch_called.borrow_mut() = true;
        *self.checked_out_branch.borrow_mut() = Some(branch.to_string());
        self.checkout_result
            .clone()
            .map_err(|e| -> Box<dyn Error> { e.into() })
    }

    fn checkout_new_branch(&self, branch: &str) -> Result<(), Box<dyn Error>> {
        *self.checkout_new_branch_called.borrow_mut() = true;
        *self.checked_out_branch.borrow_mut() = Some(branch.to_string());
        self.checkout_result
            .clone()
            .map_err(|e| -> Box<dyn Error> { e.into() })
    }

    fn commit(&self, _args: &[&str]) -> Result<String, Box<dyn Error>> {
        unimplemented!("not used in branch tests")
    }
}

async fn setup_mock(server: &MockServer, id: i64, name: &str) {
    let body = full_story_json(id, name, "description");
    Mock::given(method("GET"))
        .and(path(format!("/api/v3/stories/{id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn print_branch_name() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;
    setup_mock(&server, 123, "Fix Login Bug").await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let git_runner = TestGitRunner::new(false);
    let args = branch::BranchArgs {
        id: 123,
        prefix: None,
        checkout: false,
    };

    let result = branch::run_with_git(&args, &client, &out, &git_runner).await;
    assert!(result.is_ok());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert_eq!(output.trim(), "feature/sc-123-fix-login-bug");
}

#[tokio::test]
async fn custom_prefix() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;
    setup_mock(&server, 123, "Fix Login Bug").await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let git_runner = TestGitRunner::new(false);
    let args = branch::BranchArgs {
        id: 123,
        prefix: Some("hotfix".to_string()),
        checkout: false,
    };

    let result = branch::run_with_git(&args, &client, &out, &git_runner).await;
    assert!(result.is_ok());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert_eq!(output.trim(), "hotfix/sc-123-fix-login-bug");
}

#[tokio::test]
async fn checkout_mode() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;
    setup_mock(&server, 123, "Fix Login Bug").await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let git_runner = TestGitRunner::new(false);
    let args = branch::BranchArgs {
        id: 123,
        prefix: None,
        checkout: true,
    };

    let result = branch::run_with_git(&args, &client, &out, &git_runner).await;
    assert!(result.is_ok());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(output.contains("Created and checked out branch:"));
    assert!(output.contains("feature/sc-123-fix-login-bug"));

    let checked_out = git_runner.checked_out_branch.borrow();
    assert_eq!(checked_out.as_deref(), Some("feature/sc-123-fix-login-bug"));
    assert!(*git_runner.checkout_new_branch_called.borrow());
    assert!(!*git_runner.checkout_branch_called.borrow());
}

#[tokio::test]
async fn checkout_existing_branch() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;
    setup_mock(&server, 123, "Fix Login Bug").await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let git_runner = TestGitRunner::new(true);
    let args = branch::BranchArgs {
        id: 123,
        prefix: None,
        checkout: true,
    };

    let result = branch::run_with_git(&args, &client, &out, &git_runner).await;
    assert!(result.is_ok());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(output.contains("Checked out existing branch:"));
    assert!(output.contains("feature/sc-123-fix-login-bug"));

    let checked_out = git_runner.checked_out_branch.borrow();
    assert_eq!(checked_out.as_deref(), Some("feature/sc-123-fix-login-bug"));
    assert!(!*git_runner.checkout_new_branch_called.borrow());
    assert!(*git_runner.checkout_branch_called.borrow());
}

#[tokio::test]
async fn api_failure_returns_error() {
    let (out, _buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/999"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let git_runner = TestGitRunner::new(false);
    let args = branch::BranchArgs {
        id: 999,
        prefix: None,
        checkout: false,
    };

    let result = branch::run_with_git(&args, &client, &out, &git_runner).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn git_checkout_failure_propagates() {
    let (out, _buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;
    setup_mock(&server, 123, "Fix Login Bug").await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let git_runner = TestGitRunner::failing(false, "branch already exists");
    let args = branch::BranchArgs {
        id: 123,
        prefix: None,
        checkout: true,
    };

    let result = branch::run_with_git(&args, &client, &out, &git_runner).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("branch already exists")
    );
}
