mod support;

use sc::{api, commands::epic};
use support::{epic_json, full_epic_json};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_list_args(desc: bool) -> epic::EpicArgs {
    epic::EpicArgs {
        action: epic::EpicAction::List { desc },
    }
}

fn make_update_args(id: i64) -> epic::UpdateArgs {
    epic::UpdateArgs {
        id,
        name: None,
        description: None,
        deadline: None,
        archived: None,
        epic_state_id: None,
        labels: vec![],
        objective_ids: vec![],
        owner_ids: vec![],
        follower_ids: vec![],
        requested_by_id: None,
    }
}

#[tokio::test]
async fn list_epics_prints_names() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        epic_json(1, "Epic One", None),
        epic_json(2, "Epic Two", None),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_list_args(false);
    let result = epic::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epics_with_descriptions() {
    let server = MockServer::start().await;

    let body = serde_json::json!([epic_json(1, "Epic One", Some("Description of epic one")),]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .and(query_param("includes_description", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_list_args(true);
    let result = epic::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epics_empty() {
    let server = MockServer::start().await;

    let body = serde_json::json!([]);

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_list_args(false);
    let result = epic::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_epics_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/epics"))
        .respond_with(ResponseTemplate::new(400))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = make_list_args(false);
    let result = epic::run(&args, &client).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn update_epic_sets_description() {
    let server = MockServer::start().await;

    let body = full_epic_json(42, "My Epic", "new description");

    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.description = Some("new description".to_string());
    let args = epic::EpicArgs {
        action: epic::EpicAction::Update(Box::new(update_args)),
    };
    let result = epic::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_epic_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/v3/epics/99"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let update_args = make_update_args(99);
    let args = epic::EpicArgs {
        action: epic::EpicAction::Update(Box::new(update_args)),
    };
    let result = epic::run(&args, &client).await;
    assert!(result.is_err());
}
