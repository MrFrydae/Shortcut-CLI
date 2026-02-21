use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{
    branch_json, full_story_json, full_story_json_with_branches_and_prs, pull_request_json,
};
use sc::output::{ColorMode, OutputConfig, OutputMode};
use sc::{api, commands::story};

#[tokio::test]
async fn get_story_prints_details() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_story_json(99, "Important Story", "Some description");

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Get { id: 99 },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_story_shows_branches_and_prs() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_story_json_with_branches_and_prs(
        99,
        "Login Fix",
        "Fix the login",
        vec![branch_json(1, "feature/sc-99-login-fix", 42)],
        vec![pull_request_json(
            100,
            456,
            "Fix login bug",
            true,
            false,
            false,
        )],
    );

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Get { id: 99 },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(output.contains("Branches:"));
    assert!(output.contains("feature/sc-99-login-fix"));
    assert!(output.contains("repo 42"));
    assert!(output.contains("Pull Requests:"));
    assert!(output.contains("#456"));
    assert!(output.contains("Fix login bug"));
    assert!(output.contains("merged"));
}
