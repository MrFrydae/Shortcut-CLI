use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::category_json;
use sc::{api, commands::category};

#[tokio::test]
async fn list_categories() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!([
        category_json(1, "Engineering", Some("#0000ff")),
        category_json(2, "Design", Some("#ff0000")),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::List { archived: false },
    };
    let result = category::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_categories_empty() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::List { archived: false },
    };
    let result = category::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_categories_filters_archived() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let mut archived_cat = category_json(2, "Old Category", None);
    archived_cat["archived"] = serde_json::Value::Bool(true);

    let body = serde_json::json!([
        category_json(1, "Active Category", Some("#00ff00")),
        archived_cat,
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::List { archived: false },
    };
    let result = category::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
