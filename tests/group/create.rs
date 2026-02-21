use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{default_icon, group_json, member_json};
use crate::{UUID_ALICE, UUID_GROUP1};
use sc::{api, commands::group};

#[tokio::test]
async fn create_group_minimal() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = group_json(UUID_GROUP1, "New Team", "new-team");

    Mock::given(method("POST"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Create(Box::new(group::CreateArgs {
            name: "New Team".to_string(),
            mention_name: "new-team".to_string(),
            description: None,
            color: None,
            member_ids: vec![],
            workflow_ids: vec![],
        })),
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_group_with_members() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    // Mock members endpoint for @mention resolution
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

    let mut group = group_json(UUID_GROUP1, "Dev Team", "dev-team");
    group["member_ids"] = serde_json::json!([UUID_ALICE]);

    Mock::given(method("POST"))
        .and(path("/api/v3/groups"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&group))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = group::GroupArgs {
        action: group::GroupAction::Create(Box::new(group::CreateArgs {
            name: "Dev Team".to_string(),
            mention_name: "dev-team".to_string(),
            description: None,
            color: None,
            member_ids: vec!["@alice".to_string()],
            workflow_ids: vec![],
        })),
    };
    let tmp = tempfile::tempdir().unwrap();
    let result = group::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}
