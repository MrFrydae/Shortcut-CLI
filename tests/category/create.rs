use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::category_json;
use sc::{api, commands::category};

#[tokio::test]
async fn create_category_minimal() {
    let server = MockServer::start().await;

    let body = category_json(42, "New Category", None);

    Mock::given(method("POST"))
        .and(path("/api/v3/categories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::Create(Box::new(category::CreateArgs {
            name: "New Category".to_string(),
            color: None,
            category_type: None,
            external_id: None,
        })),
    };
    let result = category::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_category_with_color() {
    let server = MockServer::start().await;

    let body = category_json(43, "Colored Category", Some("#ff00ff"));

    Mock::given(method("POST"))
        .and(path("/api/v3/categories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::Create(Box::new(category::CreateArgs {
            name: "Colored Category".to_string(),
            color: Some("#ff00ff".to_string()),
            category_type: None,
            external_id: None,
        })),
    };
    let result = category::run(&args, &client).await;
    assert!(result.is_ok());
}
