mod support;

use sc::{api, commands::member};
use support::{default_icon, member_json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const UUID_ALICE: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const UUID_BOB: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";

fn make_args(list: bool, id: Option<&str>) -> member::MemberArgs {
    member::MemberArgs {
        list,
        id: id.map(String::from),
    }
}

#[tokio::test]
async fn list_members_prints_names() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        member_json(
            UUID_ALICE,
            "alice",
            "Alice Smith",
            "admin",
            false,
            Some(default_icon())
        ),
        member_json(UUID_BOB, "bob", "Bob Jones", "member", false, None), // null display_icon
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(true, None);
    let result = member::run(&args, &client, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_members_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(true, None);
    let result = member::run(&args, &client, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_members_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(400))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(true, None);
    let result = member::run(&args, &client, None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn get_member_by_uuid() {
    let server = MockServer::start().await;

    let body = member_json(
        UUID_ALICE,
        "alice",
        "Alice Smith",
        "admin",
        false,
        Some(default_icon()),
    );

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/members/{UUID_ALICE}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(false, Some(UUID_ALICE));
    let result = member::run(&args, &client, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_member_by_mention_name() {
    let server = MockServer::start().await;

    let list_body = serde_json::json!([
        member_json(
            UUID_ALICE,
            "alice",
            "Alice Smith",
            "admin",
            false,
            Some(default_icon())
        ),
        member_json(
            UUID_BOB,
            "bob",
            "Bob Jones",
            "member",
            false,
            Some(default_icon())
        ),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&list_body))
        .expect(1)
        .mount(&server)
        .await;

    let detail_body = member_json(
        UUID_ALICE,
        "alice",
        "Alice Smith",
        "admin",
        false,
        Some(default_icon()),
    );

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/members/{UUID_ALICE}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&detail_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(false, Some("@alice"));
    let result = member::run(&args, &client, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_member_mention_not_found() {
    let server = MockServer::start().await;

    let list_body = serde_json::json!([member_json(
        UUID_ALICE,
        "alice",
        "Alice Smith",
        "admin",
        false,
        Some(default_icon())
    ),]);

    Mock::given(method("GET"))
        .and(path("/api/v3/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&list_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(false, Some("@nobody"));
    let result = member::run(&args, &client, None).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No member found with mention name @nobody")
    );
}

#[tokio::test]
async fn get_member_invalid_id() {
    let server = MockServer::start().await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(false, Some("not-a-uuid"));
    let result = member::run(&args, &client, None).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid member ID")
    );
}

#[tokio::test]
async fn no_flags_does_nothing() {
    let server = MockServer::start().await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_args(false, None);
    let result = member::run(&args, &client, None).await;
    assert!(result.is_ok());
}
