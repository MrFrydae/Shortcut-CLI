use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{story_comment_json, story_comment_json_with_reactions, story_reaction_json};
use sc::{api, commands::story::comment as story_comment};

const UUID_AUTHOR: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

// --- List tests ---

#[tokio::test]
async fn list_story_comments() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = serde_json::json!([
        story_comment_json(1, 123, "First comment", UUID_AUTHOR),
        story_comment_json(2, 123, "Second comment", UUID_AUTHOR),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/123/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_comment::CommentArgs {
        action: story_comment::CommentAction::List { story_id: 123 },
    };
    let result = story_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_story_comments_empty() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/123/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_comment::CommentArgs {
        action: story_comment::CommentAction::List { story_id: 123 },
    };
    let result = story_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

// --- Add tests ---

#[tokio::test]
async fn add_story_comment_with_text() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = story_comment_json(10, 123, "Hello world", UUID_AUTHOR);

    Mock::given(method("POST"))
        .and(path("/api/v3/stories/123/comments"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_comment::CommentArgs {
        action: story_comment::CommentAction::Add {
            story_id: 123,
            text: Some("Hello world".to_string()),
            text_file: None,
        },
    };
    let result = story_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn add_story_comment_no_text_errors() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_comment::CommentArgs {
        action: story_comment::CommentAction::Add {
            story_id: 123,
            text: None,
            text_file: None,
        },
    };
    let result = story_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--text"));
}

// --- Get tests ---

#[tokio::test]
async fn get_story_comment() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = story_comment_json(456, 123, "A comment", UUID_AUTHOR);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/123/comments/456"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_comment::CommentArgs {
        action: story_comment::CommentAction::Get {
            story_id: 123,
            id: 456,
        },
    };
    let result = story_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_story_comment_with_reactions() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let reactions = vec![
        story_reaction_json(":thumbsup:", vec![UUID_AUTHOR]),
        story_reaction_json(":heart:", vec![UUID_AUTHOR]),
    ];
    let body =
        story_comment_json_with_reactions(456, 123, "Reacted comment", UUID_AUTHOR, reactions);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/123/comments/456"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_comment::CommentArgs {
        action: story_comment::CommentAction::Get {
            story_id: 123,
            id: 456,
        },
    };
    let result = story_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

// --- Update test ---

#[tokio::test]
async fn update_story_comment() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = story_comment_json(456, 123, "Updated text", UUID_AUTHOR);

    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/123/comments/456"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_comment::CommentArgs {
        action: story_comment::CommentAction::Update {
            story_id: 123,
            id: 456,
            text: "Updated text".to_string(),
        },
    };
    let result = story_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

// --- Delete tests ---

#[tokio::test]
async fn delete_story_comment_with_confirm() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("DELETE"))
        .and(path("/api/v3/stories/123/comments/456"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_comment::CommentArgs {
        action: story_comment::CommentAction::Delete {
            story_id: 123,
            id: 456,
            confirm: true,
        },
    };
    let result = story_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_story_comment_without_confirm_errors() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_comment::CommentArgs {
        action: story_comment::CommentAction::Delete {
            story_id: 123,
            id: 456,
            confirm: false,
        },
    };
    let result = story_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}

// --- React tests ---

#[tokio::test]
async fn react_to_story_comment() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let reaction_body = serde_json::json!([story_reaction_json(":thumbsup:", vec![UUID_AUTHOR]),]);

    Mock::given(method("POST"))
        .and(path("/api/v3/stories/123/comments/456/reactions"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&reaction_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_comment::CommentArgs {
        action: story_comment::CommentAction::React {
            story_id: 123,
            comment_id: 456,
            emoji: "thumbsup".to_string(),
        },
    };
    let result = story_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn react_already_wrapped_emoji() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let reaction_body = serde_json::json!([story_reaction_json(":heart:", vec![UUID_AUTHOR]),]);

    Mock::given(method("POST"))
        .and(path("/api/v3/stories/123/comments/456/reactions"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&reaction_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_comment::CommentArgs {
        action: story_comment::CommentAction::React {
            story_id: 123,
            comment_id: 456,
            emoji: ":heart:".to_string(),
        },
    };
    let result = story_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}

// --- Unreact test ---

#[tokio::test]
async fn unreact_from_story_comment() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("DELETE"))
        .and(path("/api/v3/stories/123/comments/456/reactions"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_comment::CommentArgs {
        action: story_comment::CommentAction::Unreact {
            story_id: 123,
            comment_id: 456,
            emoji: "thumbsup".to_string(),
        },
    };
    let result = story_comment::run(&args, &client, tmp.path()).await;
    assert!(result.is_ok());
}
