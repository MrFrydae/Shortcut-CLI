use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::objective_json;
use sc::{api, commands::category};

#[tokio::test]
async fn list_category_objectives() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        objective_json(1, "Objective A", "in progress", "First"),
        objective_json(2, "Objective B", "to do", "Second"),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/categories/42/objectives"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::Objectives {
            id: 42,
            desc: false,
        },
    };
    let result = category::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_category_objectives_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/categories/42/objectives"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::Objectives {
            id: 42,
            desc: false,
        },
    };
    let result = category::run(&args, &client).await;
    assert!(result.is_ok());
}
