use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{full_story_json, make_dry_run_output};
use sc::{api, commands::story};

#[tokio::test]
async fn delete_story_with_confirm() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_story_json(42, "My Story", "desc");

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Delete {
            id: 42,
            confirm: true,
        },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_story_without_confirm_errors() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Delete {
            id: 42,
            confirm: false,
        },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}

#[tokio::test]
async fn delete_story_not_found() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/999"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Delete {
            id: 999,
            confirm: true,
        },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn dry_run_story_delete_bypasses_confirm() {
    let (out, buf) = make_dry_run_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // No mocks — dry-run should not make any HTTP calls

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Delete {
            id: 42,
            confirm: false, // Would normally error, but dry-run bypasses
        },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(output.contains("[dry-run] DELETE /api/v3/stories/42"));
}
