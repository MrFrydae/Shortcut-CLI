use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::milestone_json;
use sc::{api, commands::category};

#[tokio::test]
async fn list_category_milestones() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        milestone_json(1, "Milestone A", "to do"),
        milestone_json(2, "Milestone B", "done"),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/categories/42/milestones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::Milestones { id: 42 },
    };
    let result = category::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_category_milestones_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/categories/42/milestones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::Milestones { id: 42 },
    };
    let result = category::run(&args, &client).await;
    assert!(result.is_ok());
}
