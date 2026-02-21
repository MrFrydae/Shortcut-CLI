use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{
    full_story_json, full_story_json_with_links, story_link_json, typed_story_link_json,
};
use sc::{api, commands::story, commands::story::link as story_link};

#[tokio::test]
async fn link_create_blocks() {
    let server = MockServer::start().await;

    let body = story_link_json(1, 10, 20, "blocks");

    Mock::given(method("POST"))
        .and(path("/api/v3/story-links"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_link::LinkArgs {
        action: story_link::LinkAction::Create(story_link::CreateLinkArgs {
            subject: 10,
            object: 20,
            verb: "blocks".to_string(),
        }),
    };
    let result = story_link::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn link_create_blocked_by_swaps() {
    let server = MockServer::start().await;

    // When verb is "blocked-by", subject/object are swapped:
    // user says "10 blocked-by 20" -> API gets "20 blocks 10"
    let body = story_link_json(2, 20, 10, "blocks");

    Mock::given(method("POST"))
        .and(path("/api/v3/story-links"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_link::LinkArgs {
        action: story_link::LinkAction::Create(story_link::CreateLinkArgs {
            subject: 10,
            object: 20,
            verb: "blocked-by".to_string(),
        }),
    };
    let result = story_link::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn link_create_relates_to() {
    let server = MockServer::start().await;

    let body = story_link_json(3, 10, 20, "relates to");

    Mock::given(method("POST"))
        .and(path("/api/v3/story-links"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_link::LinkArgs {
        action: story_link::LinkAction::Create(story_link::CreateLinkArgs {
            subject: 10,
            object: 20,
            verb: "relates-to".to_string(),
        }),
    };
    let result = story_link::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn link_list_shows_links() {
    let server = MockServer::start().await;

    let links = vec![
        typed_story_link_json(1, 42, 99, "blocks", "subject"),
        typed_story_link_json(2, 50, 42, "duplicates", "object"),
    ];
    let story_body = full_story_json_with_links(42, "My Story", "desc", links);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&story_body))
        .expect(1)
        .mount(&server)
        .await;

    // Mock fetches for linked story names
    let story_99 = full_story_json(99, "Blocked Story", "");
    Mock::given(method("GET"))
        .and(path("/api/v3/stories/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&story_99))
        .expect(1)
        .mount(&server)
        .await;

    let story_50 = full_story_json(50, "Duplicate Source", "");
    Mock::given(method("GET"))
        .and(path("/api/v3/stories/50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&story_50))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_link::LinkArgs {
        action: story_link::LinkAction::List { story_id: 42 },
    };
    let result = story_link::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn link_list_empty() {
    let server = MockServer::start().await;

    let story_body = full_story_json(42, "My Story", "desc");

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&story_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_link::LinkArgs {
        action: story_link::LinkAction::List { story_id: 42 },
    };
    let result = story_link::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn link_delete_with_confirm() {
    let server = MockServer::start().await;

    let link_body = story_link_json(5, 10, 20, "blocks");

    Mock::given(method("GET"))
        .and(path("/api/v3/story-links/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&link_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/story-links/5"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_link::LinkArgs {
        action: story_link::LinkAction::Delete {
            id: 5,
            confirm: true,
        },
    };
    let result = story_link::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn link_delete_without_confirm_errors() {
    let server = MockServer::start().await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story_link::LinkArgs {
        action: story_link::LinkAction::Delete {
            id: 5,
            confirm: false,
        },
    };
    let result = story_link::run(&args, &client).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}

#[tokio::test]
async fn get_story_with_links() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let links = vec![
        typed_story_link_json(1, 42, 99, "blocks", "subject"),
        typed_story_link_json(2, 50, 42, "relates to", "object"),
    ];
    let body = full_story_json_with_links(42, "Linked Story", "A story with links", links);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Get { id: 42 },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}
