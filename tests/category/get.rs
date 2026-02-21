use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{category_json, milestone_json, objective_json};
use sc::{api, commands::category};

#[tokio::test]
async fn get_category() {
    let server = MockServer::start().await;

    let body = category_json(42, "My Category", Some("#0000ff"));

    Mock::given(method("GET"))
        .and(path("/api/v3/categories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    // Mock milestones endpoint
    Mock::given(method("GET"))
        .and(path("/api/v3/categories/42/milestones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            milestone_json(1, "Milestone One", "in progress"),
        ])))
        .expect(1)
        .mount(&server)
        .await;

    // Mock objectives endpoint
    Mock::given(method("GET"))
        .and(path("/api/v3/categories/42/objectives"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            objective_json(1, "Objective One", "to do", "desc"),
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::Get { id: 42 },
    };
    let result = category::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_category_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/categories/999"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::Get { id: 999 },
    };
    let result = category::run(&args, &client).await;
    assert!(result.is_err());
}
