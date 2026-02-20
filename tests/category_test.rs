mod support;

use sc::{api, commands::category};
use support::{category_json, milestone_json, objective_json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// --- List tests ---

#[tokio::test]
async fn list_categories() {
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
    let result = category::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_categories_empty() {
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
    let result = category::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_categories_filters_archived() {
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
    let result = category::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Create tests ---

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

// --- Get tests ---

#[tokio::test]
async fn get_category() {
    let server = MockServer::start().await;

    let body = category_json(42, "My Category", Some("#0000ff"));

    Mock::given(method("GET"))
        .and(path("/api/v3/categories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    // Mock milestones endpoint
    Mock::given(method("GET"))
        .and(path("/api/v3/categories/42/milestones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            milestone_json(1, "Milestone One", "in progress"),
        ])))
        .expect(1)
        .mount(&server)
        .await;

    // Mock objectives endpoint
    Mock::given(method("GET"))
        .and(path("/api/v3/categories/42/objectives"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            objective_json(1, "Objective One", "to do", "desc"),
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::Get { id: 42 },
    };
    let result = category::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_category_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/categories/999"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::Get { id: 999 },
    };
    let result = category::run(&args, &client).await;
    assert!(result.is_err());
}

// --- Update test ---

#[tokio::test]
async fn update_category() {
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
    let result = category::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Delete tests ---

#[tokio::test]
async fn delete_category_with_confirm() {
    let server = MockServer::start().await;

    let body = category_json(42, "To Delete", None);

    Mock::given(method("GET"))
        .and(path("/api/v3/categories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/categories/42"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::Delete {
            id: 42,
            confirm: true,
        },
    };
    let result = category::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_category_without_confirm_errors() {
    let server = MockServer::start().await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = category::CategoryArgs {
        action: category::CategoryAction::Delete {
            id: 42,
            confirm: false,
        },
    };
    let result = category::run(&args, &client).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}

// --- Milestones listing tests ---

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

// --- Objectives listing tests ---

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
