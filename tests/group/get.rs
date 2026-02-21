use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{default_icon, group_json, member_json};
use crate::{UUID_ALICE, UUID_GROUP1, UUID_GROUP2};
use sc::{api, commands::group};

#[tokio::test]
async fn get_group() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let mut group = group_json(UUID_GROUP1, "Backend", "backend");
    group["description"] = serde_json::json!("The backend team");
    group["num_stories"] = serde_json::json!(10);
    group["num_stories_started"] = serde_json::json!(3);
    group["num_stories_backlog"] = serde_json::json!(5);
    group["num_epics_started"] = serde_json::json!(2);
    group["member_ids"] = serde_json::json!([UUID_ALICE]);

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/groups/{UUID_GROUP1}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&group))
        .expect(1)
        .mount(&server)
        .await;

    // Mock members endpoint for member info display
    let members_body = serde_json::json!([member_json(
        UUID_ALICE,
        "alice",
        "Alice Smith",
        "member",
        false,
        Some(default_icon())
    ),]);
    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&members_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Get {
            id: UUID_GROUP1.to_string(),
        },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_group_not_found() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/groups/{UUID_GROUP1}")))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Get {
            id: UUID_GROUP1.to_string(),
        },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn get_group_by_mention_name() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    // Mock list-groups for cache population (mention name resolution)
    let list_body = serde_json::json!([
        group_json(UUID_GROUP1, "Backend", "backend"),
        group_json(UUID_GROUP2, "Frontend", "frontend"),
    ]);
    Mock::given(method("GET"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&list_body))
        .expect(1)
        .mount(&server)
        .await;

    // Mock get-group for the resolved UUID
    let mut detail = group_json(UUID_GROUP1, "Backend", "backend");
    detail["description"] = serde_json::json!("The backend team");
    detail["num_stories"] = serde_json::json!(5);
    detail["num_stories_started"] = serde_json::json!(2);
    detail["num_stories_backlog"] = serde_json::json!(3);
    detail["num_epics_started"] = serde_json::json!(1);
    detail["member_ids"] = serde_json::json!([UUID_ALICE]);

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/groups/{UUID_GROUP1}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&detail))
        .expect(1)
        .mount(&server)
        .await;

    // Mock members endpoint for member info display
    let members_body = serde_json::json!([member_json(
        UUID_ALICE,
        "alice",
        "Alice Smith",
        "member",
        false,
        Some(default_icon())
    ),]);
    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&members_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Get {
            id: "@backend".to_string(),
        },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_group_by_mention_name_not_found() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let list_body = serde_json::json!([group_json(UUID_GROUP1, "Backend", "backend"),]);
    Mock::given(method("GET"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&list_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Get {
            id: "@nonexistent".to_string(),
        },
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_err());
}
