mod support;

use sc::{api, commands::search};
use support::{
    doc_slim_json, search_epic_result_json, search_iteration_result_json,
    search_objective_result_json, search_story_result_json,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_query(query: &str) -> search::SearchQueryArgs {
    search::SearchQueryArgs {
        query: query.to_string(),
        page_size: 25,
        next: None,
        desc: false,
    }
}

// --- All (unified search) tests ---

#[tokio::test]
async fn search_all_with_results() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "stories": {
            "data": [search_story_result_json(1, "Login bug", "bug")],
            "next": null,
            "total": 1,
        },
        "epics": {
            "data": [search_epic_result_json(10, "Authentication")],
            "next": null,
            "total": 1,
        },
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::All(make_query("login")),
    };
    let result = search::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn search_all_empty() {
    let server = MockServer::start().await;

    let body = serde_json::json!({});

    Mock::given(method("GET"))
        .and(path("/api/v3/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::All(make_query("nonexistent")),
    };
    let result = search::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Stories search tests ---

#[tokio::test]
async fn search_stories_with_results() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [
            search_story_result_json(1, "Login bug", "bug"),
            search_story_result_json(2, "Signup feature", "feature"),
        ],
        "next": null,
        "total": 2,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/stories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Stories(make_query("login")),
    };
    let result = search::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn search_stories_empty() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [],
        "next": null,
        "total": 0,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/stories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Stories(make_query("nothing")),
    };
    let result = search::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn search_stories_with_pagination() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [search_story_result_json(1, "Story One", "feature")],
        "next": "cursor-abc123",
        "total": 50,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/stories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Stories(make_query("story")),
    };
    let result = search::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Epics search tests ---

#[tokio::test]
async fn search_epics_with_results() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [search_epic_result_json(10, "Auth Epic")],
        "next": null,
        "total": 1,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Epics(make_query("auth")),
    };
    let result = search::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn search_epics_empty() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [],
        "next": null,
        "total": 0,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Epics(make_query("nothing")),
    };
    let result = search::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Iterations search test ---

#[tokio::test]
async fn search_iterations_with_results() {
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
    let result = search::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Milestones search test ---

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

// --- Objectives search test ---

#[tokio::test]
async fn search_objectives_with_results() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "data": [search_objective_result_json(2, "Improve Performance", "to do")],
        "next": null,
        "total": 1,
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/search/objectives"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = search::SearchArgs {
        action: search::SearchAction::Objectives(make_query("performance")),
    };
    let result = search::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Documents search tests ---

#[tokio::test]
async fn search_documents_with_results() {
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
    let result = search::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn search_documents_empty() {
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
    let result = search::run(&args, &client).await;
    assert!(result.is_ok());
}
