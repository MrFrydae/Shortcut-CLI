use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::make_query;
use crate::support::doc_slim_json;
use sc::{api, commands::search};

#[tokio::test]
async fn search_documents_with_results() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [
            doc_slim_json("00000000-0000-0000-0000-000000000001", Some("API Design Doc")),
            doc_slim_json("00000000-0000-0000-0000-000000000002", Some("Onboarding Guide")),
        ],
        "next": null,
        "total": 2,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/documents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Documents(make_query("API")),
    };
    let result = search::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn search_documents_empty() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [],
        "next": null,
        "total": 0,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/documents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Documents(make_query("nothing")),
    };
    let result = search::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
