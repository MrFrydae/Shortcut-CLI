use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::make_query;
use crate::support::search_objective_result_json;
use sc::{api, commands::search};

#[tokio::test]
async fn search_milestones_with_results() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [search_objective_result_json(1, "Q1 Goals", "in progress")],
        "next": null,
        "total": 1,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/milestones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Milestones(make_query("Q1")),
    };
    let result = search::run(&args, &client).await;
    assert!(result.is_ok());
}
