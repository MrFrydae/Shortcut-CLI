use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::group_json;
use crate::{UUID_GROUP1, UUID_GROUP2};
use sc::{api, commands::group};

#[tokio::test]
async fn list_groups() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!([
        group_json(UUID_GROUP1, "Backend", "backend"),
        group_json(UUID_GROUP2, "Frontend", "frontend"),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::List { archived: false },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_groups_empty() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::List { archived: false },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_groups_filters_archived() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let mut archived_group = group_json(UUID_GROUP2, "Old Team", "old-team");
    archived_group["archived"] = serde_json::Value::Bool(true);

    let body = serde_json::json!([
        group_json(UUID_GROUP1, "Active Team", "active-team"),
        archived_group,
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::List { archived: false },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}
