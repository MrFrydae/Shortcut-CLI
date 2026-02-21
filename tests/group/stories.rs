use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::UUID_GROUP1;
use crate::support::{group_json, story_json};
use sc::{api, commands::group};

#[tokio::test]
async fn list_group_stories() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!([
        story_json(1, "Story One", None),
        story_json(2, "Story Two", None),
    ]);

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/groups/{UUID_GROUP1}/stories")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Stories {
            id: UUID_GROUP1.to_string(),
            limit: None,
            offset: None,
            desc: false,
        },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_group_stories_empty() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/groups/{UUID_GROUP1}/stories")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Stories {
            id: UUID_GROUP1.to_string(),
            limit: None,
            offset: None,
            desc: false,
        },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn stories_by_mention_name() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    // Mock list-groups for mention name resolution
    let list_body = serde_json::json!([group_json(UUID_GROUP1, "Backend", "backend"),]);
    Mock::given(method("GET"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&list_body))
        .expect(1)
        .mount(&server)
        .await;

    // Mock stories endpoint for the resolved UUID
    let stories_body = serde_json::json!([
        story_json(1, "Story One", None),
        story_json(2, "Story Two", None),
    ]);
    Mock::given(method("GET"))
        .and(path(format!("/api/v3/groups/{UUID_GROUP1}/stories")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&stories_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Stories {
            id: "@backend".to_string(),
            limit: None,
            offset: None,
            desc: false,
        },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}
