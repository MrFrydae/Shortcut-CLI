use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::make_query;
use crate::support::search_iteration_result_json;
use sc::{api, commands::search};

#[tokio::test]
async fn search_iterations_with_results() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [search_iteration_result_json(
            1, "Sprint 1", "started",
            "2024-01-01T00:00:00Z", "2024-01-14T00:00:00Z"
        )],
        "next": null,
        "total": 1,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/iterations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Iterations(make_query("sprint")),
    };
    let result = search::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
