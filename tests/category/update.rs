use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::category_json;
use sc::{api, commands::category};

#[tokio::test]
async fn update_category() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = category_json(42, "Updated Category", Some("#00ff00"));

    Mock::given(method("PUT"))
        .and(path("/api/v3/categories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::Update(Box::new(category::UpdateArgs {
            id: 42,
            name: Some("Updated Category".to_string()),
            color: None,
            archived: None,
        })),
    };
    let result = category::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
