mod support;

use sc::{api, commands::epic_comment};
use support::threaded_comment_json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const UUID_AUTHOR: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

// --- List tests ---

#[tokio::test]
async fn list_epic_comments() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = serde_json::json!([
        threaded_comment_json(1, "First comment", UUID_AUTHOR, vec![]),
        threaded_comment_json(2, "Second comment", UUID_AUTHOR, vec![]),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic_comment::CommentArgs {
        action: epic_comment::CommentAction::List { epic_id: 42 },
    };
    let result = epic_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epic_comments_empty() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic_comment::CommentArgs {
        action: epic_comment::CommentAction::List { epic_id: 42 },
    };
    let result = epic_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epic_comments_with_threads() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let child = threaded_comment_json(3, "Reply", UUID_AUTHOR, vec![]);
    let body = serde_json::json!([threaded_comment_json(
        1,
        "Top-level comment",
        UUID_AUTHOR,
        vec![child]
    ),]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic_comment::CommentArgs {
        action: epic_comment::CommentAction::List { epic_id: 42 },
    };
    let result = epic_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

// --- Add tests ---

#[tokio::test]
async fn add_epic_comment_with_text() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = threaded_comment_json(10, "Hello epic", UUID_AUTHOR, vec![]);

    Mock::given(method("POST"))
        .and(path("/api/v3/epics/42/comments"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic_comment::CommentArgs {
        action: epic_comment::CommentAction::Add {
            epic_id: 42,
            text: Some("Hello epic".to_string()),
            text_file: None,
        },
    };
    let result = epic_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn add_epic_comment_no_text_errors() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic_comment::CommentArgs {
        action: epic_comment::CommentAction::Add {
            epic_id: 42,
            text: None,
            text_file: None,
        },
    };
    let result = epic_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--text"));
}

// --- Get tests ---

#[tokio::test]
async fn get_epic_comment() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = threaded_comment_json(100, "A comment", UUID_AUTHOR, vec![]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42/comments/100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic_comment::CommentArgs {
        action: epic_comment::CommentAction::Get {
            epic_id: 42,
            id: 100,
        },
    };
    let result = epic_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_epic_comment_with_replies() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let reply = threaded_comment_json(101, "A reply", UUID_AUTHOR, vec![]);
    let body = threaded_comment_json(100, "Parent comment", UUID_AUTHOR, vec![reply]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics/42/comments/100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic_comment::CommentArgs {
        action: epic_comment::CommentAction::Get {
            epic_id: 42,
            id: 100,
        },
    };
    let result = epic_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

// --- Update test ---

#[tokio::test]
async fn update_epic_comment() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = threaded_comment_json(100, "Updated text", UUID_AUTHOR, vec![]);

    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/42/comments/100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic_comment::CommentArgs {
        action: epic_comment::CommentAction::Update {
            epic_id: 42,
            id: 100,
            text: "Updated text".to_string(),
        },
    };
    let result = epic_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

// --- Delete tests ---

#[tokio::test]
async fn delete_epic_comment_with_confirm() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("DELETE"))
        .and(path("/api/v3/epics/42/comments/100"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic_comment::CommentArgs {
        action: epic_comment::CommentAction::Delete {
            epic_id: 42,
            id: 100,
            confirm: true,
        },
    };
    let result = epic_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_epic_comment_without_confirm_errors() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = epic_comment::CommentArgs {
        action: epic_comment::CommentAction::Delete {
            epic_id: 42,
            id: 100,
            confirm: false,
        },
    };
    let result = epic_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}
