use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::objective_json;
use sc::{api, commands::objective};

#[tokio::test]
async fn update_objective() {
    let server = MockServer::start().await;

    let body = objective_json(42, "Updated Name", "done", "Updated desc");

    Mock::given(method("PUT"))
        .and(path("/api/v3/objectives/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Update(Box::new(objective::UpdateArgs {
            id: 42,
            name: Some("Updated Name".to_string()),
            description: None,
            state: None,
            archived: None,
            categories: vec![],
        })),
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}
